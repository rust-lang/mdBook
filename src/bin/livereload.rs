extern crate websocket;
extern crate crossbeam;

use std::sync::mpsc::channel;
use std::sync::mpsc;
use std::io;
use std::thread;
use std::sync::{Arc, Mutex};
use std::ops::Deref;
use std::marker::PhantomData;

use self::websocket::header::WebSocketProtocol;
use self::websocket::ws::sender::Sender;
use self::websocket::ws::receiver::Receiver;
use self::websocket::message::Type;
use self::websocket::{Server, Message};

const WS_PROTOCOL: &'static str = "livereload";
const RELOAD_COMMAND: &'static str = "reload";


#[derive(Debug, Clone, PartialEq)]
enum MessageType {
    Reload,
    Close,
}


#[derive(Clone)]
struct ComparableSender<T> {
    sender: mpsc::Sender<T>,
    id: usize,
}

impl<T> PartialEq for ComparableSender<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Deref for ComparableSender<T> {
    type Target = mpsc::Sender<T>;

    fn deref(&self) -> &mpsc::Sender<T> {
        &self.sender
    }
}


struct ComparableSenderFactory<T> {
    next_id: usize,
    sender_type: PhantomData<T>,
}

impl<T> ComparableSenderFactory<T> {
    fn generate(&mut self, sender: mpsc::Sender<T>) -> ComparableSender<T> {
        let tx = ComparableSender {
            sender: sender,
            id: self.next_id,
        };
        self.next_id += 1;
        tx
    }

    fn new() -> ComparableSenderFactory<T> {
        ComparableSenderFactory {
            next_id: 0,
            sender_type: PhantomData,
        }
    }
}


pub struct LiveReload {
    senders: Arc<Mutex<Vec<ComparableSender<MessageType>>>>,
}

impl LiveReload {
    pub fn new(address: &str) -> io::Result<LiveReload> {
        let server = try!(Server::bind(address));

        let senders: Arc<Mutex<Vec<ComparableSender<MessageType>>>> = Arc::new(Mutex::new(vec![]));
        let senders_clone = senders.clone();

        let mut factory = ComparableSenderFactory::new();

        let lr = LiveReload { senders: senders_clone };

        // handle connection attempts on a separate thread
        thread::spawn(move || {
            for connection in server {
                let mut senders = senders.clone();
                let (tx, rx) = channel();

                let tx = factory.generate(tx);

                senders.lock().unwrap().push(tx.clone());

                // each connection gets a separate thread
                thread::spawn(move || {
                    let request = connection.unwrap().read_request().unwrap();
                    let headers = request.headers.clone();

                    let mut valid = false;
                    if let Some(&WebSocketProtocol(ref protocols)) = headers.get() {
                        if protocols.contains(&(WS_PROTOCOL.to_owned())) {
                            valid = true;
                        }
                    }

                    let client;
                    if valid {
                        let mut response = request.accept();
                        response.headers.set(WebSocketProtocol(vec![WS_PROTOCOL.to_owned()]));
                        client = response.send().unwrap();
                    } else {
                        request.fail().send().unwrap();
                        println!("{:?}", "Rejecting invalid websocket request.");
                        return;
                    }

                    let (mut ws_tx, mut ws_rx) = client.split();

                    // handle receiving and sending (websocket) in two separate threads
                    crossbeam::scope(|scope| {
                        let tx_clone = tx.clone();
                        scope.spawn(move || {
                            let tx = tx_clone;
                            loop {
                                match rx.recv() {
                                    Ok(msg) => {
                                        match msg {
                                            MessageType::Reload => {
                                                let message: Message = Message::text(RELOAD_COMMAND.to_owned());
                                                let mut senders = senders.clone();
                                                if ws_tx.send_message(&message).is_err() {
                                                    // the receiver isn't available anymore
                                                    // remove the tx from senders and exit
                                                    LiveReload::remove_sender(&mut senders, &tx);
                                                    break;
                                                }
                                            },
                                            MessageType::Close => {
                                                LiveReload::remove_sender(&mut senders, &tx);
                                                break;
                                            },
                                        }
                                    },
                                    Err(e) => {
                                        println!("{:?}", e);
                                        break;
                                    },
                                }
                            }
                        });

                        for message in ws_rx.incoming_messages() {
                            match message {
                                Ok(message) => {
                                    let message: Message = message;
                                    match message.opcode {
                                        Type::Close => {
                                            tx.send(MessageType::Close).unwrap();
                                            break;
                                        },
                                        // TODO ?
                                        // Type::Ping => {
                                        //     let message = websocket::Message::pong(message.payload);
                                        //     ws_tx.send_message(&message).unwrap();
                                        // },
                                        _ => {
                                            println!("{:?}", message.opcode);
                                            unimplemented!()
                                        },
                                    }
                                },
                                Err(err) => {
                                    println!("Error: {}", err);
                                    break;
                                },
                            }
                        }
                    });
                });
            }
        });

        Ok(lr)
    }

    fn remove_sender(senders: &mut Arc<Mutex<Vec<ComparableSender<MessageType>>>>, el: &ComparableSender<MessageType>) {
        let mut senders = senders.lock().unwrap();
        let mut index = 0;
        for i in 0..senders.len() {
            if &senders[i] == el {
                index = i;
                break;
            }
        }
        senders.remove(index);
    }

    pub fn trigger_reload(&self) {
        let senders = self.senders.lock().unwrap();
        println!("Reloading {} client(s).", senders.len());
        for sender in senders.iter() {
            sender.send(MessageType::Reload).unwrap();
        }
    }
}
