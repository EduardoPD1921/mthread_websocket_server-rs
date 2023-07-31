use std::{net::TcpStream, io::{self, Write}};

fn main() {
    let addr = format!("127.0.0.1:{}", 3000);
    let mut stream = TcpStream::connect(&addr).unwrap();

    println!("Client connected. Addr: {}", stream.local_addr().unwrap());

    let mut user_input = String::new();
    let stdin = io::stdin();

    loop {
        // TODO implement frames to dynamically allocate buffers on server side
        user_input.clear();
        stdin.read_line(&mut user_input).unwrap();

        stream.write(user_input.as_bytes()).unwrap();
    }
}
