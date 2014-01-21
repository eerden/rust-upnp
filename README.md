rust-upnp
=========

UPNP Media Server V4 implementation written in rust.
Very early stages of development. 

Currently:

- It responds to Browse actions. The library folder can be viewed.
- It can send video, and music but the metadata is not correct so you'll need a forgiving client.
- It can send subtitles if they have the same name as the video file.
- It sends ssdp:alive messages and can be discovered.

- It does not support searching, rewinding, fast forwarding or anything interesting.
- It does not respond to ssdp:discover messages. 


Requirements: 

- RustyXml for parsing incoming SOAP requests. https://github.com/Florob/RustyXML
- rustsqlite for storing/retrieving media information. https://github.com/linuxfood/rustsqlite
- A fairly new version of rust. https://github.com/mozilla/rust
- A **capable** upnp renderer. (MediaHouse on Android works. Next targets are going to be VLC, and Xbox360).

Try it:

- Compile lib.rs `rustc lib.rs -L /path/to/reqired/libs`.
- Compile test_server.rs `rustc test_server.rs -L .`
- Run it with `test_server --dir /path/to/media/directory`.

It should show up as `ZAP` on your media player's devices list.
