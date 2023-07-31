use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Arc, Mutex};

struct Client {
    stream: TcpStream,
    is_thread_active: bool
}

impl Client {
    fn new(stream: TcpStream, is_thread_active: bool) -> Client {
        Client { stream, is_thread_active  }
    }
}

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

fn watch_client_stream(mut stream: TcpStream) {
    loop {
        // let mut bytes_vec: Vec<u8> = Default::default();
        let mut msg_from_client: String = Default::default();
        match stream.read_to_string(&mut msg_from_client) {
            Ok(res) => {
                if res > 0 {
                    println!("usize: {}", res);
                }
                // println!("usize: {}", res)
                // let msg_from_client = String::from_utf8_lossy(&bytes_vec);
                // if msg_from_client.len() > 0 {
                //     println!("Message: {} from addr: {}", msg_from_client, stream.peer_addr().unwrap());
                // }
            },
            Err(e) => {
                eprintln!("Some error occurred: {}", e.to_string());
                break;
            }
        };
    }
}

fn watch_clients(clients_vec: Arc<Mutex<Vec<Client>>>) {
    let mut checks_counter = 0;
    let mut threads_counter = 0;

    loop {
        checks_counter += 1;

        let mut locked_clients_vec = clients_vec.lock().unwrap();
        if let Some(client) = locked_clients_vec.iter_mut().find(|c| c.is_thread_active == false) {
            client.is_thread_active = true;
            let client_stream_clone = client.stream.try_clone().unwrap();

            threads_counter += 1;
            thread::spawn(move || watch_client_stream(client_stream_clone));
        }

        // println!("Threads counter: {}", threads_counter);
        // println!("Checks counter: {}", checks_counter);
    }
}
