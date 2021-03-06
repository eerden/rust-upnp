use super::content_directory_v4::ContentDirectory;
use std::io::BufferedReader;
use std::io::File;
use super::ssdp;
use super::http::Request;
use std::comm::SharedChan;
use super::http;
use std::io::SeekSet;
use http::{GET,POST,HEAD};

pub struct MediaServer {
    http_addr: ~str,
    content: ~ContentDirectory ,
    from_http_chan: ~SharedChan<Request>, //TODO:Remove this
    from_http_port: ~Port<Request>, //TODO:Put this in an option.
}

impl MediaServer {
    pub fn update(&self) {
        self.content.update_db();
    }
    pub fn dispatch(&self, mut req:Request) {
        debug!("MediaServer::dispatch() : ==================START REQUEST==============");
        debug!("{}", req.to_str());
        debug!("MediaServer::dispatch() : ==================END REQUEST==============");
        let (method, url)  = (req.method.clone(), req.url.clone());
        match (method, url) {
            (GET, ~"/icon.png") => {
                spawn(proc(){
                    debug!("MediaServer:::dispatch() Icon requested.");
                    send_icon("icon.png",req);
                })
            },
            (GET, ~"/rootDesc.xml") => {
                spawn(proc(){
                    debug!("MediaServer::dispatch() : Root doc requested.");
                    send_xml_file("xml_templates/rootDesc.xml",req);
                })
            },
            (GET,~"/content_dir.xml") => {
                spawn(proc(){
                    debug!("MediaServer::dispatch() : Content directory service SCPD doc requested.");
                    send_xml_file("xml_templates/content_dir.xml",req);
                })
            },

            (GET,~"/connection_manager.xml") => {
                spawn(proc(){
                    debug!("MediaServer::dispatch() : connection_manager.xml requested.");
                    send_xml_file("xml_templates/connection_manager.xml",req);
                })
            },

            (GET,~"/X_MS_MediaReceiverRegistrar.xml") => {
                spawn(proc(){
                    debug!("MediaServer::dispatch() : X_MS_media_receiver_registrar.xml  requested.");
                    send_xml_file("xml_templates/X_MS_media_receiver_registrar.xml",req);
                })
            },

            (POST,~"/control/content_dir") => {
                debug!("MediaServer::dispatch() : Content directory service control command.");
                self.content.browse(req);
            }
            (POST, _) => {
                req.stream.write(http::code_404());
            }

            (GET, _) => {
                self.send_video(req);
            }
            (HEAD, _ ) => {
                req.stream.write(http::default_vid_headers());
            }
        }
    }

    pub  fn new( addr: &str, library_dir: &str) -> MediaServer {
        let cd = ~ContentDirectory::new(library_dir.to_owned());
        let (port, chan) : (Port<Request>,SharedChan<Request>) = SharedChan::new();
        MediaServer{content: cd, http_addr: addr.to_owned(), from_http_chan: ~chan, from_http_port: ~port}
    }

    pub fn up(&mut self) {
        ssdp::advertise(self.get_messages());
        http::listen(self.http_addr.clone(), self.from_http_chan.clone());
        println!("MediaServer: Server up.");
        loop {
            debug!("MediaServer::up() : New request received from from_http_chan");
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
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 Zap/1.1.1\r
NT:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911\r
NTS:ssdp:alive\r\n\r\n"

);


        out.push(
            ~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 Zap/1.1.1\r
NT:upnp:rootdevice\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::upnp:rootdevice\r
NTS:ssdp:alive\r\n\r\n"
);

        out.push (
            ~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 Zap/1.1.1\r
NT:urn:schemas-upnp-org:device:MediaServer:1\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:schemas-upnp-org:device:MediaServer:1\r
NTS:ssdp:alive\r\n\r\n"
);


        out.push (

            ~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 Zap/1.1.1\r
NT:urn:schemas-upnp-org:service:ContentDirectory:4\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:schemas-upnp-org:service:ContentDirectory:4\r
NTS:ssdp:alive\r\n\r\n"
);

        out.push (

            ~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 Zap/1.1.1\r
NT:urn:schemas-upnp-org:service:ConnectionManager:1\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:schemas-upnp-org:service:ConnectionManager:1\r
NTS:ssdp:alive\r\n\r\n"
);


        out.push(
            ~"NOTIFY * HTTP/1.1\r
HOST:239.255.255.250:1900\r
CACHE-CONTROL:max-age=20\r
LOCATION:http://192.168.1.3:8900/rootDesc.xml\r
SERVER: 3.12.1-3-ARCH DLNADOC/1.50 UPnP/1.0 Zap/1.1.1\r
NT:urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1\r
USN:uuid:4d696e69-444c-164e-9d41-e0cb4ebb5911::urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1\r
NTS:ssdp:alive\r\n\r\n"
);

        out 
    }

    fn send_video(&self, mut request: Request) {
        debug!("MediaServer::send_video() : Video requested.");
        //'/MediaItems/[id].avi'
        let vid_path = match self.content.get_item_path(request.url.clone()) {
            None    => {
                request.stream.write(http::code_404()); 
                return
            }, //This is failure
            Some(p) => p
        };


        spawn(proc(){
            let mut file = File::open(&vid_path);
            let file_length = match vid_path.stat() {
                Ok(stat)    => stat.size,
                Err(e)      => fail!("Error getting file information for file: {}. Error: {}", vid_path.display(), e)
            };

            let buf = BufferedReader::new(file);
            let content_length_header = ("Content-Length: " + file_length.to_str() + "\r\n\r\n").into_bytes();
            let mut request = request;
            let mut buf = buf;
            request.stream.write(http::default_vid_headers());
            request.stream.write(content_length_header);
            loop {
                match buf.read_byte() {
                    Ok(b)   => match request.stream.write_u8(b) {
                        Ok(res) => (),
                        Err(e)  => fail!("Can't write byte to stream. Error: {}", e)
                    },
                    Err(e)  => fail!("Can't read byte from file buffer. Error: {}", e)
                }
            }
        })
    }
}

fn send_xml_file(filename: &str, mut request: Request) {
    let path = Path::new("./" + filename);
    let mut file = File::open(&path);
    let buf = match file.read_to_end() {
        Ok(b)   => b,
        Err(e)  => fail!("Can't read file : {}. Error: {}", path.display(), e.to_str())
    };

    let content_length_header = ("Content-Length: " + buf.len().to_str() + "\r\n\r\n").into_bytes();
    request.stream.write(http::default_xml_headers());
    request.stream.write(content_length_header);
    request.stream.write(buf);
}

fn send_icon(filename: &str, mut request: Request) {
    let path = Path::new("./" + filename);
    let mut file = File::open(&path);
    let buf = match file.read_to_end() {
        Ok(b)   => b,
        Err(e)  => fail!("Can't read file : {}. Error: {}", path.display(), e.to_str())
    };

    let content_length_header = ("Content-Length: " + buf.len().to_str() + "\r\n\r\n").into_bytes();
    request.stream.write(http::default_img_headers());
    request.stream.write(content_length_header);
    request.stream.write(buf);
}
