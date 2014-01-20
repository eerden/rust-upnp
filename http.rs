use std::comm::SharedChan;
use std::hashmap::HashMap;
use std::io::Acceptor;
use std::io::Listener;
use std::io::net::tcp::TcpListener;
use std::io::net::tcp::TcpStream;
use std::str;

pub fn listen (addr: &str, server_chan: ~SharedChan<Request>) {
    let address = addr.to_owned();
    do spawn {
        let socket_addr = from_str(address).unwrap();
        let listener = TcpListener::bind(socket_addr);
        let mut acceptor = listener.listen().unwrap();
        loop {
            let stream = match acceptor.accept(){
                Some(s) => s,
                None    => fail!("Can't get a TcpStream from the std::io::Acceptor!")
            };
            let  chan = server_chan.clone();
            do spawn {
                let request = Request::new(stream);
                chan.try_send(request);
            }
        }
    }
}

pub struct Request {
    method: Method,
    url:    ~str,
    http_info: ~str,
    headers: HashMap<~str,~str>,
    body: Option<~str>,
    stream: TcpStream
}

impl Request {
    fn new(mut stream: TcpStream) -> Request {
        let mut header_lines = Request::get_header_lines(&mut stream);
        let method_line = header_lines.shift();
        let (method, url, http_info) = Request::method_url_and_http(method_line);
        let headers = Request::get_headers(header_lines);
        let body = match method {
            POST => {
                let len_str = (headers.get(&~"Content-Length"));
                let len : int = from_str(*len_str).unwrap();
                Request::get_body(&mut stream,len)
            }
            _    => None
        };
        Request{method: method, url: url, http_info: http_info, headers:headers, body: body, stream: stream}
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

    //Returns a ~str array contatining header lines.
    //This stops at the point where '\r\n\r\n' is reached.
    fn get_header_lines(stream: &mut TcpStream) -> ~[~str] {
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

//Use for debugging.
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


//TODO: This is here just to make things work for the moment. Find a better way of doing this.
pub fn default_xml_headers() -> ~[u8] {
    let out :~str = ~"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: text/xml; charset=\"utf-8\"\r\n";
    out.into_bytes()
}

//TODO: This is here just to make things work for the moment. Find a better way of doing this.
pub fn default_img_headers() -> ~[u8] {
    let out :~str = ~"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: image/png\r\n";
    out.into_bytes()
}

//TODO: This is here just to make things work for the moment. Find a better way of doing this.
pub fn default_vid_headers() -> ~[u8] {
    let out :~str = ~"HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Type: video/mp4\r\n";
    out.into_bytes()

}
pub fn code_404() -> ~[u8] {
    let out :~str = ~"HTTP/1.1 404 Not Found\r\n\r\n";
    out.into_bytes()
}

