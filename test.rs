extern mod upnp;
use upnp::http::Request;
use std::io::net::tcp::TcpStream;
use std::io::{File,fs};
use std::comm::Chan;

#[test]
fn test_http(){
    let c = advertise();
    do spawn{
        upnp::http::listen("192.168.1.3:8900",handle);
    }
}

fn handle(req:Request) -> ~[u8]{
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

        }
        //(POST,~"/control/connection_manager") => println("Connection manager service control command."),
        //(POST,~"/control/av_transport") => println("AV transport service control command."),

        //(POST,~"/event/content_dir") => println("Content directory service event command."),
        //(POST,~"/event/connection_manager") => println("Connection manager service event command."),
        //(POST,~"/event/av_transport") => println("AV transport service event command."),

        (_,_) => send_empty()
    }
}
fn content_dir(req: Request) -> [u8] {

}

fn send_xml_file(filename: &str, req: Request) -> ~[u8]{
    let mut response : ~[u8] = ~[];
    let xml_headers = upnp::http::default_xml_headers();
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    response.push_all_move(xml_headers);
    response.push_all_move(buf);
    response
}

fn send_icon(filename: &str, req: Request) -> ~[u8] {
    let mut response : ~[u8] = ~[];
    let img_headers = upnp::http::default_img_headers();
    let path = Path::new("/home/ercan/rust/src/upnp/" + filename);
    let mut file = File::open(&path);
    let buf = file.read_to_end();
    response.push_all_move(img_headers);
    response.push_all_move(buf);
    response

}

fn advertise() -> std::comm::Chan<bool>{
   let messages = upnp::device::media_server_v4::get_messages(); 
   let kill_chan = upnp::ssdp::advertise(messages);
   kill_chan
}

fn send_empty() -> ~[u8]{
    ~[]
}
