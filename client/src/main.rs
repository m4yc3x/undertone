use clap::{App, Arg};
use tokio::signal;
use tokio::select;

mod utility;
mod networking;

#[tokio::main]
async fn main() {
    utility::log("Undertone client is starting...");
    let matches = App::new("Undertone Client")
        .version("version 0.1.0 (testing)")
        .arg(Arg::with_name("connect")
            .short("c")
            .long("connect")
            .value_name("ID")
            .help("Connect to a peer"))
        .arg(Arg::with_name("secure")
            .short("s")
            .long("secure")
            .takes_value(false)
            .help("Enable longer identifiers and extra encryption"))
        .get_matches();

    let secure_mode = matches.is_present("secure");
    if secure_mode {
        utility::log("Secure mode enabled. Longer identifiers and extra encryption will be used");
    } else {
        utility::log("Secure mode not enabled. Standard encryption will be used.");
    }

    utility::log("Connecting to signaling server...");

    let connect_to = matches.value_of("connect").unwrap_or("disabled").to_string();

    // Wrap the main logic in an async block passed to `select!`
    let main_logic = async {
        if let Err(e) = networking::bootstrap::connect(connect_to, secure_mode).await {
            utility::err(&format!("Failed to connect: {}", e));
        }
    };

    // Wait for either the main logic to complete or a Ctrl+C signal
    select! {
        _ = main_logic => {
            // Main logic has completed
            utility::log("Main logic completed.");
        },
        _ = signal::ctrl_c() => {
            // Ctrl+C signal received
            utility::log("Termination received. Exiting gracefully...");
            // Perform any necessary cleanup here
        },
    }
}
