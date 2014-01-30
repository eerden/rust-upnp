#[comment = "Rust UPNP library."];
#[crate_type = "lib"];
#[crate_id = "upnp#0.1"];

extern mod xml;
extern mod sqlite3;

pub mod content_directory_v4;
pub mod connection_manager_v3;
pub mod av_transport_v3;
pub mod media_server_v4;
pub mod ssdp;
pub mod http;
pub mod template;
pub mod magic;
pub mod result_item;
