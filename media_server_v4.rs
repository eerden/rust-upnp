use super::content_directory_v4::ContentDirectory;
use super::connection_manager_v3::ConnectionManager;
use super::av_transport_v3::AvTransport;
use std::io::{File,fs};
use super::ssdp;
use super::http::Request;
use std::comm::SharedChan;
use std::comm::Chan;
use super::http;
use std::io::SeekSet;
use std::io::stdio::println;
pub struct MediaServer {
    //av_trans : ~AvTransport,
    //conn_man : ~ConnectionManager,
    http_addr: ~str,
    cont_dir : ~ContentDirectory ,
    //TODO: See if these make any sense
    //This is given to the http server.
    from_http_chan: ~SharedChan<Request>, //TODO:Remove this
    //This gets messages from the http server.
    from_http_port: ~Port<Request>, //TODO:Put this in an option.

}

impl MediaServer {
    pub  fn new(desc_xml: &str, addr: &str) -> MediaServer {
        let cd = ~ContentDirectory{service_reset_token: ~"12345"};
        let (port, chan) : (Port<Request>,SharedChan<Request>) = SharedChan::new();
        MediaServer{cont_dir: cd, http_addr: addr.to_owned(), from_http_chan: ~chan, from_http_port: ~port}
    }

    pub fn up(&mut self) {
        let ssdp_kill_chan = ssdp::advertise(self.get_messages());
        http::listen(self.http_addr.clone());
        println("Server up.");
    }

    pub fn get_messages(&self) -> ~[~str]{
        let mut out :~[~str] = ~[];
        out.push(
            ~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 MiniDLNA/1.1.1\r
NT:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911\r
NTS:ssdp:alive\r\n");

        out.push(
            ~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8200/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 MiniDLNA/1.1.1\r
NT:urn:schemas-upnp-org:service:ContentDirectory:1\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:schemas-upnp-org:service:ContentDirectory:4\r
NTS:ssdp:alive\r\n");
        out 
    }
}

pub fn get_messages() -> ~[~str]{
    let mut out :~[~str] = ~[];
    out.push(
        ~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 MiniDLNA/1.1.1\r
NT:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911\r
NTS:ssdp:alive\r\n");


    out.push(
        ~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8200/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 MiniDLNA/1.1.1\r
NT:urn:schemas-upnp-org:service:ContentDirectory:1\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:schemas-upnp-org:service:ContentDirectory:4\r
NTS:ssdp:alive\r\n");
    out 
}

fn handle(req:Request) -> ~[u8]{
    println("handle function called...");
    println(req.to_str());
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
        //(GET,~"/connection_manager.xml") => println("Connection manager service SCPD doc requested."),
        //(GET,~"/av_transport.xml") => println("AV Transport service SCPD doc requested."),

        (POST,~"/control/content_dir") => {
            println("Content directory service control command.");
            ContentDirectory::browse(req)
        }

        (GET, url) => {
            send_video(req)

        }

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
    let mut url = req.url.slice_from(12);
    println("what");
    let dot_pos = match url.find('.') {
        Some(pos)   => pos,
        None        => fail!("Can't find the '.' in file name")
    };

    println(url);
    println(dot_pos.to_str());
    let id : int = match from_str(url.slice_to(dot_pos)) {
        Some(num)   => num,
        None        => fail!("Can't make an int from id string.")
    };

    let vid_path = Path::new(ContentDirectory::get_item_url(id));



    println("ID:" + id.to_str());

    let mut start : i64 = 0;
    match req.headers.find_copy(&~"Range") {
        None => (),
        Some(r) => {
            start = get_byte_range(r);
        },
    }

    let mut response : ~[u8] = ~[];
    let img_headers = http::default_img_headers();
    //let path = Path::new("/home/ercan/StreamMedia/Series/South Park/Season 17/S17E01 - Let Go Let Gov.mp4");
    let mut file = File::open(&vid_path);
    file.seek(start, SeekSet);
    let buf = file.read_to_end();
    response.push_all_move(img_headers);
    response.push_all_move(buf);
    response

}


fn send_xml_file(filename: &str, req: Request) -> ~[u8]{
    let mut response : ~[u8] = ~[];
    let xml_headers = http::default_xml_headers();
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    response.push_all_move(xml_headers);
    response.push_all_move(buf);
    response
}

fn send_icon(filename: &str, req: Request) -> ~[u8] {
    let mut response : ~[u8] = ~[];
    let img_headers = http::default_img_headers();
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    response.push_all_move(img_headers);
    response.push_all_move(buf);
    response
}

//TODO: Make this an HTTP error message.
fn send_empty() -> ~[u8]{
    ~[]
}


