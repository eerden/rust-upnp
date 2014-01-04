use std::io::net::tcp::TcpStream;
use std::str;
pub use std::io::net::tcp::TcpListener;
use std::io::Listener;
use std::io::Acceptor;
use std::hashmap::HashMap;
use std::comm::SharedChan;
use std::io::timer::Timer;

pub fn listen (addr: &str, func: fn(Request) -> ~[u8]){
    let socket_addr = from_str(addr).unwrap();
    let mut listener = TcpListener::bind(socket_addr);
    let mut acceptor = listener.listen().unwrap();
    loop {
        let stream = match acceptor.accept(){
            Some(s) => s,
            None    => fail!("Can't get a TcpStream from the std::io::Acceptor!")
        };
        do spawn{
            let mut timer = Timer::new().unwrap();
            let (request, mut stream) = build_request(stream);
            let response = func(request);
            println(response.len().to_str());
            stream.write(response);
        }
    }
}

fn build_request(mut stream: TcpStream) -> (Request, TcpStream){
    let (mut header_lines, stream) = get_header_lines(stream);
    let method_line = header_lines.shift();
    let (method, url, http_info) = get_method_url_http(method_line);
    let headers = get_headers(header_lines);
    let (body,stream) = match method {
        POST => {
            let len_str = (headers.get(&~"Content-Length"));
            let len : int = from_str(*len_str).unwrap();
            get_body(stream,len)
        }
        _    => (None,stream)
    };
    (Request{method: method, url: url, http_info: http_info, headers:headers, body: body},stream)
}

//Extracts the http method, url and http version information from the first line of an http request.
fn get_method_url_http(line: ~str) -> (Method,~str,~str){
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

fn get_headers(lines : ~[~str]) -> HashMap<~str,~str>{
    let mut headers : HashMap<~str,~str> = HashMap::new();
    for line in lines.iter() {
        let split_line : ~[&str] = line.splitn(':',1).collect();
        headers.insert( split_line[0].trim().to_owned(), split_line[1].trim().to_owned());
    }
    headers
}

fn get_body(mut stream: TcpStream, len: int) -> (Option<~str>,TcpStream){
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
    (Some(str::from_utf8(out).to_owned()),stream)
}

//Returns the stream back and the  header part of the requests as a ~str array.
//This stops at the point where '\r\n\r\n' is reached.
fn get_header_lines(mut stream: TcpStream) -> (~[~str], TcpStream){
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
    (out, stream)
}

pub struct Request {
    method: Method,
    url:    ~str,
    http_info: ~str,
    headers: HashMap<~str,~str>,
    body: Option<~str>
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
pub fn default_xml_headers() -> ~[u8]{
    let out :~str = ~"HTTP/1.1 200 OK\r\nContent-Type: text/xml; charset=\"utf-8\"\r\n\r\n";
    out.into_bytes()

}

pub fn default_img_headers() -> ~[u8]{
    let out :~str = ~"HTTP/1.1 200 OK\r\nContent-Type: image/png\r\n\r\n";
    out.into_bytes()

}
