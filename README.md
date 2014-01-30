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
- CMake version 2.8 or higher. Take a look at https://github.com/SiegeLord/RustCMake if you want to use CMake with rust.
- A fairly new version of rust. https://github.com/mozilla/rust
- A **capable** upnp renderer. (MediaHouse on Android works. Next targets are going to be VLC, and Xbox360).

Try it:

- Clone the repository with `--recursive` to get the submodules: `git clone --recursive https://github.com/eerden/rust-upnp.git`
- In rust-upnp folder: 
    mkdir build
    cd build
    cmake ..
    make test_server
    cd bin
    ./test_server --dir /path/to/your/media/directory

It should show up as `ZAP` on your media player's devices list with a red 'Z' icon.
