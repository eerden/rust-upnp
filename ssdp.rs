use std::io::net::udp::UdpSocket;
use std::io::stdio::println;
use std::io::net::ip::SocketAddr;
use std::io::timer::Timer;
use std::comm::Port;
use std::comm::Chan;
use std::comm::{TryRecvResult,Data};

pub fn advertise(messages: ~[~str]) -> Chan<bool> {
    let (kill_port, kill_chan) = Chan::<bool>::new();
    do spawn  {
        let socket_addr = from_str("239.255.255.250:1900").unwrap();
        let mut socket = UdpSocket::bind(socket_addr).unwrap(); 
        let mut timer = Timer::new().unwrap();
        loop {
            match kill_port.try_recv() {
                Data(val) => {
                    println("ssdp::advertise() : Received kill message, ending task.");
                    break;
                }
                _         =>{
                    for m in messages.iter() {
                        timer.sleep(10);
                        socket.sendto(m.as_bytes(), socket_addr);
                    }

                }

            }
            timer.sleep(3000);
        }
    }
    kill_chan
}

pub fn listen() -> Chan<bool>{
    let ip = from_str("239.255.255.250").unwrap();
    let port = from_str("1900").unwrap();
    let socket = UdpSocket::bind(SocketAddr{ip: ip , port: port}).unwrap(); 
    let (kill_port, kill_chan) = Chan::<bool>::new();

    do spawn{
        let mut a = 1u64;
        let mut stream = socket.connect(SocketAddr{ip: ip , port: port});
        let mut timer = Timer::new().unwrap();
        loop {
            match kill_port.try_recv() {
                Data(val) => {
                    println("ssdp::listen() : Received kill message, ending task.");
                    break;
                }
                _____      => {

                    println("Trying to read from stream.");
                    let buf = stream.read_byte();
                    println("Read one byte from stream.");
                    println(buf.unwrap().to_str());
                    timer.sleep(1);
                    a += 1;

                }
            }

        }
    }

    kill_chan
}
