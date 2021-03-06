use std::io::net::udp::UdpSocket;
use std::io::stdio::println;
use std::io::timer::Timer;

pub fn advertise(messages: ~[~str]) {
    spawn(proc(){
        let socket_addr = from_str("239.255.255.250:1900").unwrap();
        let mut socket = UdpSocket::bind(socket_addr).unwrap(); 
        let mut timer = Timer::new().unwrap();
        loop {
            timer.sleep(5000);
            for m in messages.iter() {
                timer.sleep(10);
                socket.sendto(m.as_bytes(), socket_addr);
            }
            for m in messages.iter() {
                timer.sleep(10);
                socket.sendto(m.as_bytes(), socket_addr);
            }
        }
    })
}

//This is non-functional at the moment.
pub fn listen() {
    spawn(proc(){
        let socket_addr = from_str("239.255.255.250:1900").unwrap();
        let socket = UdpSocket::bind(socket_addr).unwrap(); 
        let mut stream = socket.connect(socket_addr);
        loop {
            println("Trying to read from stream.");
            let buf = stream.read_byte();
            println("Read one byte from stream.");
            println(buf.unwrap().to_str());
        }
    })
}
