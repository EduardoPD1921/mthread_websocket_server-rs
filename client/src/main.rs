use std::net::TcpStream;
use std::io::{self, Write, BufReader, BufRead};
use std::thread;
use std::env;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    data: Vec<u8>,
    user_name: String
}

impl Message {
    fn new(data: Vec<u8>, user_name: String) -> Message {
        Message { data, user_name }
    }
}

fn main() {
    let addr = format!("127.0.0.1:{}", 3000);
    let mut stream = TcpStream::connect(&addr).unwrap();
    let stream_clone = stream.try_clone().unwrap();

    let vec_args: Vec<String> = env::args().collect();

    println!("Client connected. Addr: {}", stream.local_addr().unwrap());

    thread::spawn(move || watch_server_messages(stream_clone));

    let mut user_input = String::new();
    let stdin = io::stdin();

    loop {
        let user_name = vec_args.get(1).unwrap().to_owned();

        user_input.clear();
        stdin.read_line(&mut user_input).unwrap();

        let data = user_input.as_bytes().to_vec();
        let message = Message::new(data, user_name);

        let serialized_message = serde_json::to_string(&message).unwrap();
        writeln!(stream, "{}", serialized_message).unwrap();
    }
}

fn watch_server_messages(stream: TcpStream) {
    loop {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut message_string: String = Default::default();

        match reader.read_line(&mut message_string) {
            Ok(0) => {
                println!("Error 500");
                // todo!();
                continue;
            },
            Ok(_) => {
                if message_string != "\n" {
                    let message: Message = serde_json::from_str(&message_string).unwrap();
                    let mut client_msg = String::from_utf8_lossy(&message.data).to_string();
                    client_msg.pop();

                    println!("{}: {}", message.user_name, client_msg);
                }
            },
            Err(_e) => {
                // todo!();
                continue;
            }
        }
    }
}
