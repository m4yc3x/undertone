// Importing necessary libraries
use std::sync::Arc;
use clap::{App, Arg};
use tokio::signal;
use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use futures::sink::SinkExt;
use std::collections::HashMap;
use futures::stream::StreamExt;
use tokio_tungstenite::accept_async;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use chrono::Local;
use colored::*;

// Defining the structure of the UTPacket
#[derive(Serialize, Deserialize, Debug)]
struct Message {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    id: Option<String>,
    to: Option<String>,
    from: Option<String>,
    payload: Option<String>,
}

// Main function
#[tokio::main]
async fn main() {

    log("Undertone server is starting...");

    // Parsing command line arguments
    let matches = App::new("Undertone Server")
        .version("version 0.1.0 (testing)")
        .author("--------------------------------")
        .about("A signaling server based on the UTPacket architecture")
        .arg(Arg::with_name("port")
             .long("port")
             .takes_value(true)
             .help("Sets the port to listen on"))
        .arg(Arg::with_name("host")
             .long("host")
             .takes_value(true)
             .help("Sets the IP address to listen on"))
        .arg(Arg::with_name("log")
             .short('L')
             .long("log")
             .takes_value(false)
             .help("Enable logging to the console"))
        .arg(Arg::with_name("debug")
             .short('D')
             .long("debug")
             .takes_value(false)
             .help("Enable debug messages"))
        .arg(Arg::with_name("help")
             .short('?')
             .long("help")
             .takes_value(false)
             .help("Prints help information"))
        .get_matches();

    // Getting the port and host from the command line arguments
    let port = matches.value_of("port").unwrap_or("1720");
    let host = matches.value_of("host").unwrap_or("0.0.0.0");
    let addr = format!("{}:{}", host, port);

    let log_enabled = matches.is_present("log");
    let debug_enabled = matches.is_present("debug");

    // Checking if logging is enabled
    if matches.is_present("log") {
        log(&format!("Logging is enabled!"));
    } else {
        log(&format!("Logging is disabled!"));
    }

    // Creating a TCP listener
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    log(&format!("Server listening on ws://{}", addr));

    // Creating a HashMap to store connections
    let connections = Arc::new(Mutex::new(HashMap::new()));

    // Accepting incoming connections
    let main_logic = async {
        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream.peer_addr().expect("connected streams should have a peer address");
            if matches.is_present("log") {
                log(&format!("> New peer address: {}", peer));
            }

            // Accepting the websocket handshake
            let ws_stream = accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");

            // Creating a channel for the connection
            let (tx, rx) = mpsc::channel(32);
            connections.lock().await.insert(peer.to_string(), tx);

            // Spawning a new task to handle the connection
            let connections_clone = connections.clone();
            tokio::spawn(handle_connection(ws_stream, peer.to_string(), connections_clone, rx, log_enabled, debug_enabled));
        }
    };

    select! {
        _ = main_logic => {
            // Main logic has completed
            log("Main logic completed.");
        },
        _ = signal::ctrl_c() => {
            // Ctrl+C signal received
            log("Termination received. Exiting gracefully...");
            
        },
    }
}

// Function to handle a connection
async fn handle_connection(
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    client_id: String,
    connections: Arc<Mutex<HashMap<String, mpsc::Sender<String>>>>,
    mut rx: mpsc::Receiver<String>,
    log_enabled: bool,
    debug_enabled: bool, 
) {
    let mut registered_id: Option<String> = None;

    while let Some(message) = ws_stream.next().await {
        match message {
            Ok(msg) => {
                if msg.is_text() || msg.is_binary() {
                    let msg_text = msg.into_text().unwrap();
                    let parsed_msg: Result<Message, _> = serde_json::from_str(&msg_text);

                    if debug_enabled {
                        match &parsed_msg {
                            Ok(parsed) => {
                                // Debug print the parsed_msg values
                                log(&format!("Parsed Message: {:?}", parsed));
                            },
                            Err(e) => {
                                log(&format!("Failed to parse message: {}", e));
                                continue; // Skip this iteration of the loop if parsing fails
                            }
                        }
                    }

                    if let Ok(parsed_msg) = parsed_msg {
                        match parsed_msg.msg_type.as_deref() {
                            Some("register") => {
                                if let Some(id) = &parsed_msg.id {
                                    registered_id = Some(id.clone());
                                    // Only insert the sender part into the connections HashMap
                                    if let Some(tx) = {
                                        let connections_lock = connections.lock().await;
                                        connections_lock.get(&client_id).cloned()
                                    } {
                                        // If the client is already registered, replace the sender
                                        let mut connections_lock = connections.lock().await;
                                        connections_lock.insert(id.clone(), tx);
                                    } else {
                                        // This is a new registration
                                        let mut connections_lock = connections.lock().await;
                                        if let Some(tx) = connections_lock.remove(&client_id) {
                                            connections_lock.insert(id.clone(), tx);
                                        }
                                    }

                                    if log_enabled {
                                        log(&format!("> Client {} registered", id));
                                    }

                                    // Create a new message with msg_type "hello" and send it back to the client
                                    let hello_message = Message {
                                        msg_type: Some("hello".to_string()),
                                        id: None,
                                        to: Some(id.clone()),
                                        from: None,
                                        payload: None,
                                    };
                                    let hello_message_str = serde_json::to_string(&hello_message).unwrap();
                                    if let Ok(_msg) = ws_stream.send(WsMessage::Text(hello_message_str)).await {
                                        if log_enabled {
                                            log(&format!("> Hello message sent to Client {}", id));
                                        }
                                    }
                                }
                            },
                            _ => {
                                if let Some(to) = &parsed_msg.to {
                                    if let Some(tx) = connections.lock().await.get(to) {
                                        let _ = tx.send(msg_text).await;

                                        if log_enabled {
                                            log(&format!("> Message relayed to Client {}", to));
                                        }
                                    } else {
                                        if log_enabled {
                                            log(&format!("> Client {} not found", to));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if !e.to_string().contains("Connection reset without closing handshake") {
                    err(&format!("Error: {}", e));
                }
                break;
            },
        }

        // Sending messages from the channel
        while let Ok(msg) = rx.try_recv() {
            if let Err(e) = ws_stream.send(WsMessage::Text(msg)).await {
                if log_enabled {
                    err(&format!("Failed to send message: {}", e));
                }
                break;
            }
        }
    }

    if let Some(id) = registered_id {
        let mut connections_lock = connections.lock().await;
        connections_lock.remove(&id);
        if log_enabled {
            log(&format!("> Client {} unregistered at {}", id, Local::now()));
        }
    }
}

fn log(message: &str) {
    let timestamp = Local::now();
    println!("[{}] {}", timestamp.format("%Y-%m-%d %H:%M:%S").to_string().yellow(), message.cyan());
}

fn err(message: &str) {
    let timestamp = Local::now();
    println!("[{}] [{}] {}", timestamp.format("%Y-%m-%d %H:%M:%S").to_string().yellow(), "ERROR".to_string().red(), message.cyan());
}