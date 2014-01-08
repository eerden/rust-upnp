use super::content_directory_v4::ContentDirectory;
use super::connection_manager_v3::ConnectionManager;
use super::av_transport_v3::AvTransport;

struct MediaServer {
    //av_trans : ~AvTransport,
    //conn_man : ~ConnectionManager,
    cont_dir : ~ContentDirectory 
}


fn new(desc_xml: &str) -> MediaServer {
    let cd = ~ContentDirectory{service_reset_token: ~"12345"};
    MediaServer{cont_dir: cd}
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
