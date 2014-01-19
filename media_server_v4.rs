use super::content_directory_v4::ContentDirectory;
use std::io::buffered::BufferedReader;
use super::connection_manager_v3::ConnectionManager;
use super::av_transport_v3::AvTransport;
use std::io::{File};
use super::ssdp;
use super::http::Request;
use std::comm::SharedChan;
use std::comm::Chan;
use super::http;
use std::io::SeekSet;

pub struct MediaServer {
    http_addr: ~str,
    content: ~ContentDirectory ,
    from_http_chan: ~SharedChan<Request>, //TODO:Remove this
    from_http_port: ~Port<Request>, //TODO:Put this in an option.
}

impl MediaServer {

    pub fn dispatch(&self, req:Request) {
        debug!("dispatch function called...");
        debug!("==================START REQUEST==============");
        debug!("{}", req.to_str());
        debug!("==================END REQUEST==============");
        let (method, url)  = (req.method.clone(), req.url.clone());
        match (method, url) {
            (GET, ~"/icon.png") => {
                do spawn    {
                    debug!("Icon requested.");
                    send_icon("icon.png",req);
                }
            },
            (GET, ~"/rootDesc.xml") => {
                do spawn    {
                    debug!("Root doc requested.");
                    send_xml_file("rootDesc.xml",req);
                }
            },
            (GET,~"/content_dir.xml") => {
                do spawn    {
                    debug!("Content directory service SCPD doc requested.");
                    send_xml_file("content_dir.xml",req);
                }
            },

            (POST,~"/control/content_dir") => {
                debug!("Content directory service control command.");
                self.content.browse(req);
            }

            (GET, _) => {
                do spawn    {
                    send_video(req);
                }
            }
        }
    }

    pub  fn new(desc_xml: &str, addr: &str) -> MediaServer {
        let cd = ~ContentDirectory::new();
        let (port, chan) : (Port<Request>,SharedChan<Request>) = SharedChan::new();
        MediaServer{content: cd, http_addr: addr.to_owned(), from_http_chan: ~chan, from_http_port: ~port}
    }

    pub fn up(&mut self) {
        ssdp::advertise(self.get_messages());
        http::listen(self.http_addr.clone(), self.from_http_chan.clone());
        println!("Server up.");
        loop {
            match self.from_http_port.recv_opt() {
                Some(r) => self.dispatch(r),
                None    => ()
            }
        }
    }

    //TODO: Fix this mess.
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
NTS:ssdp:alive\r\n\r\n"

);


out.push(
~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 MiniDLNA/1.1.1\r
NT:upnp:rootdevice\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::upnp:rootdevice\r
NTS:ssdp:alive\r\n\r\n"
);

out.push (
~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 MiniDLNA/1.1.1\r
NT:urn:schemas-upnp-org:device:MediaServer:1\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:schemas-upnp-org:device:MediaServer:1\r
NTS:ssdp:alive\r\n\r\n"

);


out.push (

~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 MiniDLNA/1.1.1\r
NT:urn:schemas-upnp-org:service:ContentDirectory:4\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:schemas-upnp-org:service:ContentDirectory:4\r
NTS:ssdp:alive\r\n\r\n"

);

out.push (

~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 MiniDLNA/1.1.1\r
NT:urn:schemas-upnp-org:service:ConnectionManager:1\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:schemas-upnp-org:service:ConnectionManager:1\r
NTS:ssdp:alive\r\n\r\n"

);


out.push(
~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 MiniDLNA/1.1.1\r
NT:urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1\r
NTS:ssdp:alive\r\n\r\n"
);

out 
    }

}


//TODO: This is horrible. For every video MediaHouse tries things like videoname.{srt,txt...}.
//Send a proper 404 msg.
//Details are at https://tools.ietf.org/html/rfc2616#section-3.12 and 
//https://tools.ietf.org/html/rfc2616#section-14.35
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

fn send_xml_file(filename: &str, mut req: Request) {
    debug!("XML requested.");
    let mut response : ~[u8] = ~[];
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    let content_length_header = ("Content-Length: " + buf.len().to_str() + "\r\n\r\n").into_bytes();
    debug!("{}",::std::str::from_utf8(response));
    req.stream.write(http::default_xml_headers());
    req.stream.write(content_length_header);
    req.stream.write(buf);
}

fn send_icon(filename: &str, mut req: Request) {
    debug!("Icon requested.");
    let mut response : ~[u8] = ~[];
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    let content_length_header = ("Content-Length: " + buf.len().to_str() + "\r\n\r\n").into_bytes();
    req.stream.write(http::default_img_headers());
    req.stream.write(content_length_header);
    req.stream.write(buf);
}


fn send_video(mut req: Request) {
    debug!("Video requested.");
    //'/MediaItems/[id].avi'
    if req.url.len() < 12 { return }
    let mut url = req.url.slice_from(12);
    let dot_pos = match url.find('.') {
        Some(pos)   => pos,
        None        => fail!("Can't find the '.' in file name")
    };

    println!("URL : `{}`",url);
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

    let mut file = File::open(&vid_path);
    file.seek(start, SeekSet);
    let pos = file.tell();
    debug!("Start position: {} ", pos.to_str());
    let file_length = ::std::io::fs::stat(&vid_path).size;
    let content_length = file_length - pos;
    let mut buf = BufferedReader::new(file);
    let content_length_header = ("Content-Length: " + content_length.to_str() + "\r\n\r\n").into_bytes();
    req.stream.write(http::default_vid_headers());
    req.stream.write(content_length_header);
    loop {
        match buf.read_byte() {
            Some(b) => req.stream.write_u8(b),
            None    => break
        }
    }
}


struct Config {
    name: ~str

}


