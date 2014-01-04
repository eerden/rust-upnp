use service::content_directory_v4::ContentDirectory;
use service::connection_manager_v3::ConnectionManager;
use service::av_transport_v3::AvTransport;

struct MediaServer {
    av_trans : ~AvTransport,
    conn_man : ~ConnectionManager,
    cont_dir : ~ContentDirectory 
}

impl MediaServer {

}
