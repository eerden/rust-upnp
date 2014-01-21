extern mod upnp;
use upnp::media_server_v4::MediaServer;

fn main(){
    let mut library_dir = ~"";
    let addr = "192.168.1.3:8900";
    let args = std::os::args();

    if args.len() > 2 {
        match (args[1].as_slice(),args[2]) {
            ("--dir", b) => {
                if std::path::Path::new(b.as_slice()).is_dir() {
                    library_dir = b;
                }
            },
                _       => ()
        }
    } else {
        println!("\nYou have to supply a media directory using '--dir DIRNAME'\n");
        return;
    }
    let mut server = MediaServer::new(addr, library_dir);
    server.update(); //Tell the server to scan the media directory.
    server.up();
}
