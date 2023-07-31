use std::{net::TcpStream, io::{self, Write}};
use serde::Serialize;

#[derive(Serialize, Debug)]
struct Message {
    data: Vec<u8>
}

impl Message {
    fn new(data: Vec<u8>) -> Message {
        Message { data }
    }
}

fn main() {
    let addr = format!("127.0.0.1:{}", 3000);
    let mut stream = TcpStream::connect(&addr).unwrap();

    println!("Client connected. Addr: {}", stream.local_addr().unwrap());

    let mut user_input = String::new();
    let stdin = io::stdin();

    loop {
        user_input.clear();
        stdin.read_line(&mut user_input).unwrap();

        let message_data = user_input.as_bytes().to_vec();
        let frame = Message::new(message_data);

        let serialized_message = serde_json::to_string(&frame).unwrap();
        writeln!(stream, "{}", serialized_message).unwrap();
    }
}
