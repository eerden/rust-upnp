use std::io::net::tcp::TcpStream;
use std::str;
pub use std::io::net::tcp::TcpListener;
use std::io::Listener;
use std::io::Acceptor;
use std::hashmap::HashMap;

pub fn listen(addr: &str){
    let socket_addr = from_str(addr).unwrap();
    let mut listener = TcpListener::bind(socket_addr);
    let mut acceptor = listener.listen().unwrap();
    loop {
        let mut stream   = acceptor.accept();
        process_request(stream.unwrap());
    }
}

fn process_request(mut stream: TcpStream){
    let request = build_request(stream);
    println("\n" + request.to_str());
}

fn build_request(mut stream: TcpStream) -> Request{
    let (mut header_lines, mut stream) = get_header_lines(stream);
    let method_line = header_lines.shift();
    let (method,url,http_info) = get_method_url_http(method_line);
    let headers = get_headers(header_lines);
    //let h2 : HashMap<~str,~str> = HashMap::new();
    
    let body = match method {
        POST => get_body(stream,6),
        _    => None
    };

    Request{method: method, url: url, http_info: http_info, headers:headers, body: body}
}

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

//fn get_headers(lines : ~[~str]) -> ~[~Header]{
    //let mut out : ~[~Header] = ~[];
    //for line in lines.iter() {
        //let split_line : ~[&str] = line.splitn(':',1).collect();
        //out.push(~Header{name: split_line[0].to_owned(), val: split_line[1].to_owned()});
    //}
    //out
//}

fn get_headers(lines : ~[~str]) -> HashMap<~str,~str>{
    let mut headers : HashMap<~str,~str> = HashMap::new();
    for line in lines.iter() {
        let split_line : ~[&str] = line.splitn(':',1).collect();
        headers.insert( split_line[0].to_owned(), split_line[1].to_owned());
    }
    headers
}

fn get_body(mut stream: TcpStream, len: int) -> Option<~str>{
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
        if counter == 6 {
            break;
        }
    }
    Some(str::from_utf8(out).to_owned())
}

//Get the header part of the requests one byte array.
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

    let str_blob = str::from_utf8(line);
    for l in str_blob.split('\n') {
        if l.len() > 0 { //Discard 
            out.push(l.to_owned());
        }
    }
    (out, stream)
}

struct Request {
    method: Method,
    url:    ~str,
    http_info: ~str,
    headers: HashMap<~str,~str>,
    body: Option<~str>
}

impl ToStr for Request{
    fn to_str(&self) -> ~str{
        let met  = "METHOD    : " + self.method.to_str() + "\n";
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
