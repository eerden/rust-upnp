#[comment = "Rust UPNP library."];
//#[crate_type = "lib"];
#[crate_id = "upnp#0.1"];

extern mod extra;

pub mod service;
pub mod device;
pub mod ssdp;
pub mod http;
