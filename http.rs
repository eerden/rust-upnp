extern mod extra;
use std::io::net::tcp::TcpStream;
use std::io::SeekSet;
use std::io::{File,fs};
use std::io::stdio::println;
use super::content_directory_v4::ContentDirectory;
use std::str;
pub use std::io::net::tcp::TcpListener;
use std::io::Listener;
use std::io::Acceptor;
use std::hashmap::HashMap;
use std::comm::SharedChan;
use std::comm::Chan;
use std::io::timer::Timer;
use super::media_server_v4::MediaServer;
use std::sync::arc::UnsafeArc;

use extra::arc::MutexArc;

pub fn listen (addr: &str) {
    let address = addr.to_owned();
    do spawn {
        let socket_addr = from_str(address).unwrap();
        let mut listener = TcpListener::bind(socket_addr);
        let mut acceptor = listener.listen().unwrap();
        loop {

            let stream = match acceptor.accept(){
                Some(s) => s,
                None    => fail!("Can't get a TcpStream from the std::io::Acceptor!")
            };

            do spawn {
                gogo(stream);
            }
        }
    }
}

fn gogo(mut s: TcpStream) {
    loop {
        println("Loopy------------------------------------------------------------------------------------------");
        let request = Request::new(&mut s);
        let response = handle(request);
        println!("---writing response, length: {} bytes", response.len());
        s.write(response);
        println("---done writing response");
    }
}

fn get_byte_range(rstr: &str) -> i64{
    //bytes=[start]-[end]
    //bytes=[start]- for to the end
    let mut out : i64 = 0;
    let s = rstr.trim().slice_from(6);;
    let dash_pos = match s.find('-') {
        Some(pos)   => pos,
        None        => fail!("Can't find the '-' in Range header")

    };
    if s.len() > dash_pos + 1 {
        //more after '-'

    } else { 
        let intstr = s.slice_to(dash_pos );
        out = match from_str(intstr) {
            Some(bytes)   => bytes,
            None        => fail!("Can't find the '-' in Range header")
        };
    }

    out
}

fn send_video(req: Request) -> ~[u8] {
    println("Video requested.");
    //'/MediaItems/[id].avi'
    if req.url.len() < 12 { return ~[]}
    let mut url = req.url.slice_from(12);
    let dot_pos = match url.find('.') {
        Some(pos)   => pos,
        None        => fail!("Can't find the '.' in file name")
    };

    let id : int = match from_str(url.slice_to(dot_pos)) {
        Some(num)   => num,
        None        => fail!("Can't make an int from id string.")
    };

    let vid_path = Path::new(ContentDirectory::get_item_url(id));

    let mut start : i64 = 0;
    match req.headers.find_copy(&~"Range") {
        None => (),
        Some(r) => {
            start = get_byte_range(r);
        },
    }

    let mut response : ~[u8] = ~[];
    let img_headers = default_img_headers();
    //let path = Path::new("/home/ercan/StreamMedia/Series/South Park/Season 17/S17E01 - Let Go Let Gov.mp4");
    let mut file = File::open(&vid_path);
    file.seek(start, SeekSet);
    let buf = file.read_to_end();

    let content_length_header = ("Content-Length: " + buf.len().to_str() + "\r\n\r\n").into_bytes();
    response.push_all_move(img_headers);
    response.push_all_move(content_length_header);
    response.push_all_move(buf);
    response

}
fn send_xml_file(filename: &str, req: Request) -> ~[u8]{
    let mut response : ~[u8] = ~[];
    let xml_headers = default_xml_headers();
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    let content_length_header = ("Content-Length: " + buf.len().to_str() + "\r\n\r\n").into_bytes();
    response.push_all_move(xml_headers);
    response.push_all_move(content_length_header);
    response.push_all_move(buf);
    println(::std::str::from_utf8(response));
    response
}

fn send_icon(filename: &str, req: Request) -> ~[u8] {
    let mut response : ~[u8] = ~[];
    let img_headers = default_img_headers();
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    let content_length_header = ("Content-Length: " + buf.len().to_str() + "\r\n\r\n").into_bytes();
    response.push_all_move(img_headers);
    response.push_all_move(content_length_header);
    response.push_all_move(buf);
    response
}


fn handle(req:Request) -> ~[u8]{
    println("handle function called...");
    println("==================START REQUEST==============");
    println(req.to_str());
    println("==================END REQUEST==============");
    let (method, url)  = (req.method.clone(), req.url.clone());
    match (method, url) {
        (GET, ~"/icon.png") => {
            println("Icon requested.");
            send_icon("icon.png",req)
        },
        (GET, ~"/rootDesc.xml") => {
            println("Root doc requested.");
            send_xml_file("rootDesc.xml",req)
        },
        (GET,~"/content_dir.xml") => {
            println("Content directory service SCPD doc requested.");
            send_xml_file("content_dir.xml",req)
        },

        (POST,~"/control/content_dir") => {
            println("Content directory service control command.");
            ContentDirectory::browse(req)
        }

        (GET, _) => {
            send_video(req)
        }

        (POST, _) => {
            fail!("This is not cool.");
        }

    }
}


pub struct Request {
    method: Method,
    url:    ~str,
    http_info: ~str,
    headers: HashMap<~str,~str>,
    body: Option<~str>
}

impl Request {
    fn new(mut stream: &mut TcpStream) -> Request {
        let mut header_lines = Request::get_header_lines(stream);
        let method_line = header_lines.shift();
        let (method, url, http_info) = Request::method_url_and_http(method_line);
        let headers = Request::get_headers(header_lines);
        let body = match method {
            POST => {
                let len_str = (headers.get(&~"Content-Length"));
                let len : int = from_str(*len_str).unwrap();
                Request::get_body(stream,len)
            }
            _    => None
        };
        Request{method: method, url: url, http_info: http_info, headers:headers, body: body}
    }


    //Extracts the http method, url and http version information from the first line of an http request.
    fn method_url_and_http(line: ~str) -> (Method,~str,~str){
        //Request-Line   = Method SP Request-URI SP HTTP-Version CRLF
        let parts : ~[&str] = line.split(' ').collect();

        let method : Method = match from_str(parts[0]) {
            None => fail!("Can't parse method verb"),
            Some(m) => m
        };
        let url = parts[1].to_owned();
        let http_info = parts[2].to_owned();
        (method,url,http_info)
    }

    //Returns the stream back and the  header part of the requests as a ~str array.
    //This stops at the point where '\r\n\r\n' is reached.
    fn get_header_lines(mut stream: &mut TcpStream) -> ~[~str] {
        let mut got_rn = false;
        let mut line : ~[u8] = ~[];
        let mut out : ~[~str] = ~[];
        loop {
            match stream.read_byte(){
                None => (),
                Some(b) if b as char =='\r' => (),
                Some(b) if b as char == '\n' && got_rn == true => {
                    break;
                },
                Some(b) if b as char == '\n' => {
                    got_rn = true;
                    line.push(b);
                },
                Some(b) => {
                    got_rn = false;
                    line.push(b);
                }
            }
        }

        //
        let str_blob = str::from_utf8(line);
        for l in str_blob.split('\n') {
            if l.len() > 0 { //Discard 
                out.push(l.to_owned());
            }
        }
        out
    }

    fn get_body(stream: &mut TcpStream, len: int) -> Option<~str> {
        let mut out : ~[u8] = ~[];
        let mut counter = 0;
        loop {
            match stream.read_byte() {
                None    => break,
                Some(b) => {
                    out.push(b);
                }

            }
            counter += 1;
            if counter == len {
                break;
            }
        }
        Some(str::from_utf8(out).to_owned())
    }

    fn get_headers(lines : ~[~str]) -> HashMap<~str,~str>{
        let mut headers : HashMap<~str,~str> = HashMap::new();
        for line in lines.iter() {
            let split_line : ~[&str] = line.splitn(':',1).collect();
            headers.insert( split_line[0].trim().to_owned(), split_line[1].trim().to_owned());
        }
        headers
    }



}

impl ToStr for Request{
    fn to_str(&self) -> ~str{
        let met  = "\nMETHOD    : " + self.method.to_str() + "\n";
        let url  = "URL       : " + self.url + "\n";
        let http = "HTTP INFO: " + self.http_info +"\n";
        let mut headers_string = ~"Headers:\n-------";

        let body  = match self.body {
            Some(ref s) => ~"\nBODY     : " + s.clone(),
            None    => ~"\nNO_BODY_DATA"
        };

        for (k,v) in self.headers.iter(){
            headers_string.push_str("\n" + *k + " : " + *v);
        }
        met+url+http+headers_string+body
    }
}

#[deriving(Clone)]
enum Method {
    GET,
    POST,
}

impl ToStr for Method{
    fn to_str(&self) -> ~str{
        match *self {
            GET => ~"GET",
            POST => ~"POST",

        }
    }
}

impl FromStr for Method {
    fn from_str(s: &str) -> Option<Method>{
        match s {
            "GET"   => Some(GET),
            "POST"  => Some(POST),
            _       => None
        }
    }
}

impl  Eq for Method {
    fn eq(&self, m: &Method) -> bool{
        match (self, m) {
            (&POST,&POST) => true,
            (&GET,&GET) => true,
            _       => false
        }
    }

    fn ne(&self, m: &Method) -> bool{
        true
    }
}

//TODO: This is here just to make things work for the moment. Find a better way of doing this.
pub fn default_xml_headers() -> ~[u8]{
    let out :~str = ~"HTTP/1.1 200 OK\r\nConnection: Keep-Alive\r\nContent-Type: text/xml; charset=\"utf-8\"\r\n";
    out.into_bytes()

}

//TODO: This is here just to make things work for the moment. Find a better way of doing this.
pub fn default_img_headers() -> ~[u8]{
    let out :~str = ~"HTTP/1.1 200 OK\r\nConnection: Keep-Alive\r\nContent-Type: image/png\r\n";
    out.into_bytes()

}
//TODO: Make this an HTTP error message.
fn send_empty() -> ~[u8]{
    ~[]
}
