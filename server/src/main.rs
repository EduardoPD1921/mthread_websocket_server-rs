use std::io::{BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};

use serde::Deserialize;

struct Client {
    stream: TcpStream,
    is_thread_active: bool
}

impl Client {
    fn new(stream: TcpStream, is_thread_active: bool) -> Client {
        Client { stream, is_thread_active  }
    }
}

#[derive(Deserialize, Debug)]
struct Message {
    data: Vec<u8>
}

static mut THREADS_COUNTER: i32 = 0;

fn main() {
    let addr = format!("127.0.0.1:{}", 3000);
    let listener = TcpListener::bind(&addr).unwrap();
    println!("Listening to: {}", addr);

    let clients_vec: Arc<Mutex<Vec<Client>>> = Arc::new(Mutex::new(Vec::new()));
    let clients_vec_push_connections = Arc::clone(&clients_vec);
    let clients_watch_vec = Arc::clone(&clients_vec);

    let thread_push_connections = thread::spawn(move || receive_client_connection(listener, clients_vec_push_connections));
    let thread_factory = thread::spawn(move || watch_clients(clients_watch_vec));

    thread_push_connections.join().unwrap();
    thread_factory.join().unwrap();
}

fn receive_client_connection(listener: TcpListener, clients_vec: Arc<Mutex<Vec<Client>>>) {
    for stream_result in listener.incoming() {
        let stream = match stream_result {
            Ok(stream) => stream,
            Err(e) => {
                println!("Connection error. Detailed error: {}", e.to_string());
                continue;
            }
        };

        let client_addr = stream.peer_addr().unwrap();
        println!("Client connected with the address: {}", client_addr);

        let new_client = Client::new(stream, false);
        clients_vec.lock().unwrap().push(new_client);
    }
}

fn watch_client_stream(stream: TcpStream, clients_vec: Arc<Mutex<Vec<Client>>>) {
    unsafe {
        THREADS_COUNTER += 1;
        println!("Threads counter: {}", THREADS_COUNTER);
    }

    loop {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut message_string: String = Default::default();

        match reader.read_line(&mut message_string) {
            Ok(0) => {
                println!("User {} disconnected", stream.peer_addr().unwrap());
                disconnect_client(stream, clients_vec);
                break;
            },
            Ok(_) => {
                let message: Message = serde_json::from_str(&message_string).unwrap();
                let client_msg = String::from_utf8_lossy(&message.data);

                println!("Message: {} from addr: {}", client_msg, stream.peer_addr().unwrap());
            },
            Err(e) => {
                eprintln!("Some error occurred: {}", e.to_string());
                break;
            }
        }
    }
}

fn disconnect_client(stream: TcpStream, clients_vec: Arc<Mutex<Vec<Client>>>) {
    let mut locked_clients_vec = clients_vec.lock().unwrap();

    let client_index = locked_clients_vec.iter().position(|c| c.stream.peer_addr().unwrap() == stream.peer_addr().unwrap()).unwrap();
    locked_clients_vec.remove(client_index);

    unsafe {
        THREADS_COUNTER -= 1;
        println!("Threads counter: {}", THREADS_COUNTER);
    }
}

fn watch_clients(clients_vec: Arc<Mutex<Vec<Client>>>) {
    loop {
        let mut locked_clients_vec = clients_vec.lock().unwrap();
        if let Some(client) = locked_clients_vec.iter_mut().find(|c| c.is_thread_active == false) {
            client.is_thread_active = true;
            let client_stream_clone = client.stream.try_clone().unwrap();

            let clients_vec_clone = Arc::clone(&clients_vec);
            thread::spawn(move || watch_client_stream(client_stream_clone, clients_vec_clone));
        }
    }
}
