use std::net::TcpStream;
use std::io::{self, Write, BufReader, BufRead};
use std::thread;
use std::sync::{mpsc, Arc, Mutex, Condvar};
use std::sync::mpsc::{Sender, Receiver};
use std::env;

use serde::{Serialize, Deserialize};
use pancurses::{initscr, noecho, Input};

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

enum NcurseActionType {
    WriteClientMessage,
    GetClientInput
}

struct NcurseAction {
    action_type: NcurseActionType,
    user_name: Option<String>,
    data: Option<String>
}

impl NcurseAction {
    fn new(action_type: NcurseActionType, user_name: Option<String>, data: Option<String>) -> NcurseAction {
        NcurseAction { action_type, user_name, data }
    }
}

fn main() {
    let addr = format!("127.0.0.1:{}", 3000);
    let mut stream = TcpStream::connect(&addr).unwrap();
    let stream_clone = stream.try_clone().unwrap();

    let vec_args: Vec<String> = env::args().collect();

    println!("Client connected. Addr: {}", stream.local_addr().unwrap());

    let (tx, rx): (Sender<NcurseAction>, Receiver<NcurseAction>) = mpsc::channel();
    let tx_thread = tx.clone();

    let user_input_vec: Arc<(Mutex<Vec<char>>, Condvar)> = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
    let user_input_vec_ncurse_thread = Arc::clone(&user_input_vec);

    thread::spawn(move || ncurses_thread(rx, user_input_vec_ncurse_thread));
    thread::spawn(move || watch_server_messages(stream_clone, tx_thread));

    loop {
        let user_name = vec_args.get(1).unwrap().to_owned();

        let get_client_input_action = NcurseAction::new(NcurseActionType::GetClientInput, None, None);
        match tx.send(get_client_input_action) {
            Ok(_) => {
                let (lock, cvar) = &*user_input_vec;
                let locked_user_input_vec = lock.lock().unwrap();
                let locked_user_input_vec = cvar.wait(locked_user_input_vec).unwrap();

                let user_input_string = locked_user_input_vec.iter().collect::<String>();

                let data = user_input_string.as_bytes().to_vec();
                let message = Message::new(data, user_name.clone());

                let serialized_message = serde_json::to_string(&message).unwrap();
                writeln!(stream, "{}", serialized_message).unwrap();

                let write_client_message_action = NcurseAction::new(NcurseActionType::WriteClientMessage, Some("You".to_string()), Some(user_input_string));
                tx.send(write_client_message_action).unwrap();
            },
            Err(_e) => {}
        }
    }
}

// TODO pass the user_input_vec through NcurseActionType
fn ncurses_thread(thread_rx: Receiver<NcurseAction>, user_input_vec: Arc<(Mutex<Vec<char>>, Condvar)>) {
    let window = initscr();
    noecho();
    window.keypad(true);

    loop {
        let ncurse_action = thread_rx.recv().unwrap();
        match ncurse_action.action_type {
            NcurseActionType::WriteClientMessage => {
                let formatted_message = format!("{}: {}", ncurse_action.user_name.unwrap(), ncurse_action.data.unwrap());

                window.mv(window.get_cur_y(), 0);
                window.addstr(formatted_message);
                window.addch('\n');
            },
            // TODO fix: this action is blocking the entire ncurse_thread
            NcurseActionType::GetClientInput => {
                let (lock, cvar) = &*user_input_vec;
                let mut locked_user_input_vec = lock.lock().unwrap();
                locked_user_input_vec.clear();

                loop {
                    let user_char = window.getch().unwrap();
                    match user_char {
                        Input::Character('\n') => {
                            break;
                        },
                        Input::KeyBackspace => {
                            window.mv(window.get_cur_y(), window.get_cur_x() - 1);
                            window.delch();
                            locked_user_input_vec.pop();
                        },
                        Input::Character(c) => {
                            locked_user_input_vec.push(c);
                            window.addch(c);
                        },
                        _ => {}
                    }
                }

                cvar.notify_one();
            }
        }
    }
}

fn watch_server_messages(stream: TcpStream, tx_thread: Sender<NcurseAction>) {
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

                    let write_client_message_action = NcurseAction::new(NcurseActionType::WriteClientMessage, Some(message.user_name), Some(client_msg));
                    tx_thread.send(write_client_message_action).unwrap();
                }
            },
            Err(_e) => {
                // todo!();
                continue;
            }
        }
    }
}
