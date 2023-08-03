use std::net::TcpStream;
use std::io::{Write, BufReader, BufRead};
use std::thread;
use std::sync::mpsc::{self, Sender, Receiver};
use std::env;

use serde::{Serialize, Deserialize};
use pancurses::{initscr, noecho, Input, newwin, endwin};

#[derive(Serialize, Deserialize, Debug)]
struct SocketMessage {
    data: Vec<u8>,
    username: String
}

impl SocketMessage {
    fn new(data: Vec<u8>, username: String) -> SocketMessage {
        SocketMessage { data, username }
    }
}

struct LocalMessage {
    content: String,
    username: Option<String>
}

impl LocalMessage {
    fn new(content: String, username: Option<String>) -> LocalMessage {
        LocalMessage { content, username }
    }
}

fn main() {
    let addr = format!("127.0.0.1:{}", 3000);
    let stream = TcpStream::connect(&addr).unwrap();
    let ti_thread_stream = stream.try_clone().unwrap();

    println!("Client connected. Addr: {}", stream.local_addr().unwrap());

    let (tx, rx): (Sender<LocalMessage>, Receiver<LocalMessage>) = mpsc::channel();
    let text_input_thread_tx = tx.clone();
    let network_thread_tx = tx.clone();

    let main_window_size = get_main_window_size();

    let ui_thread = thread::spawn(move || ui_thread(rx));
    let text_input_thread = thread::spawn(move || text_input_thread(main_window_size, text_input_thread_tx, ti_thread_stream));
    let network_thread = thread::spawn(move || watch_server_messages(stream, network_thread_tx));

    network_thread.join().unwrap();
    ui_thread.join().unwrap();
    text_input_thread.join().unwrap();
}

fn ui_thread(thread_rx: Receiver<LocalMessage>) {
    let ui_window = initscr();
    noecho();
    ui_window.keypad(true);

    loop {
        let local_message = thread_rx.recv().unwrap();

        let username = match local_message.username {
            Some(username) => username,
            None => String::from("You")
        };

        let formatted_message = format!("{}: {}", username, local_message.content);

        ui_window.mv(ui_window.get_cur_y(), 0);
        ui_window.addstr(formatted_message);
        ui_window.addch('\n');

        ui_window.refresh();
    }
}

fn text_input_thread(main_window_yx: (i32, i32), thread_tx: Sender<LocalMessage>, mut stream: TcpStream) {
    let text_input_window_height = 1;
    let text_input_window_width = main_window_yx.1;
    let text_input_window_start = main_window_yx.0 - 1;

    let text_input_window = newwin(text_input_window_height, text_input_window_width, text_input_window_start, 0);
    text_input_window.keypad(true);
    noecho();

    let mut text_input_vec: Vec<char> = Vec::new();

    loop {
        loop {
            match text_input_window.getch().unwrap() {
                Input::Character('\n') => {
                    text_input_window.deleteln();
                    text_input_window.mv(0, 0);
                    break;
                },
                Input::Character(c) => {
                    text_input_vec.push(c);
                    text_input_window.addch(c);
                },
                Input::KeyBackspace => {
                    text_input_window.mv(text_input_window.get_cur_y(), text_input_window.get_cur_x() - 1);
                    text_input_window.delch();
                    text_input_vec.pop();
                },
                _ => {}
            }
        }

        let vec_args: Vec<String> = env::args().collect();
        let username = vec_args.get(1).unwrap();

        let user_input_string = text_input_vec.iter().collect::<String>();
        let user_input_bytes = user_input_string.as_bytes().to_vec();
        text_input_vec.clear();

        let socket_message = SocketMessage::new(user_input_bytes, username.to_string());
        let serialized_socket_message = serde_json::to_string(&socket_message).unwrap();

        let local_message = LocalMessage::new(user_input_string, None);

        writeln!(stream, "{}", serialized_socket_message).unwrap();
        thread_tx.send(local_message).unwrap();
    }
}

fn get_main_window_size() -> (i32, i32) {
    let window = initscr();
    let yx = window.get_max_yx();
    endwin();

    yx
}

fn watch_server_messages(stream: TcpStream, thread_tx: Sender<LocalMessage>) {
    loop {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut message_string: String = Default::default();

        match reader.read_line(&mut message_string) {
            Ok(0) => {
                todo!();
            },
            Ok(_) => {
                if message_string != "\n" {
                    let socket_message: SocketMessage = serde_json::from_str(&message_string).unwrap();
                    let client_msg = String::from_utf8_lossy(&socket_message.data).to_string();

                    let local_message = LocalMessage::new(client_msg, Some(socket_message.username));
                    thread_tx.send(local_message).unwrap();
                }
            },
            Err(_e) => {
                todo!();
            }
        }
    }
}
