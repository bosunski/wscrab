use std::io::Write;
use clap::{Command, Parser, error::ErrorKind, CommandFactory};
use regex::Regex;
use colored::*;
use websocket::sender::Sender;
use websocket::ws::dataframe::DataFrame;

use std::io::stdin;
use std::sync::mpsc::{channel, Sender as ChannelSender};
use std::thread;

use websocket::client::ClientBuilder;
use websocket::{Message, OwnedMessage};

use rustyline::error::ReadlineError;
use rustyline::{Editor, Result as LineResult};

/// Just another port of wscat to Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Args {
   /// WebSocket URL to connect to
   #[arg(short, long)]
   connect: Option<String>,
}

fn main() -> LineResult<()> {
    let args = Args::parse();

    let mut cmd = Args::command();
    let url = String::from(args.connect.as_deref().unwrap());
    let re = Regex::new(r"^\w+://.*$").unwrap();

    let mut modified_url = String::from("ws://");
    modified_url.push_str(&url);

    let url = if re.is_match(&url) {url} else {
        modified_url
    };

    println!("Connection URL is {}!", url);

    if ! url.starts_with("ws") {
        cmd.error(ErrorKind::InvalidValue, "The URL must start with ws:// or wss://").exit();
    }

    println!("Connecting to {}", url);

	let client = ClientBuilder::new(&url)
		.unwrap()
		.connect_insecure()
		.unwrap();

	println!("{}", "Connected (press CTRL+C to quit)".green());

	let (mut receiver, mut sender) = client.split().unwrap();

	let (tx, rx) = channel();

	let tx_1 = tx.clone();

	let send_loop = thread::spawn(move || {
		loop {
			// Send loop
			let message = match rx.recv() {
				Ok(m) => m,
				Err(e) => {
					println!("Send Loop: {:?}", e);
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					let _ = sender.send_message(&message);
					// If it's a close message, just send it and then return.
					return;
				}
				_ => (),
			}
			// Send the message
			match sender.send_message(&message) {
				Ok(()) => (),
				Err(e) => {
					println!("Send Loop: {:?}", e);
					let _ = sender.send_message(&Message::close());
					return;
				}
			}
		}
	});

	let receive_loop = thread::spawn(move || {
		// Receive loop
		for message in receiver.incoming_messages() {
			let message = match message {
				Ok(m) => m,
				Err(e) => {
					println!("Receive Loop: {:?}", e);
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
			};
			match message {
				OwnedMessage::Close(_) => {
					// Got a close message, so send a close message and return
					let _ = tx_1.send(OwnedMessage::Close(None));
					return;
				}
				OwnedMessage::Ping(data) => {
					match tx_1.send(OwnedMessage::Pong(data)) {
						// Send a pong in response
						Ok(()) => (),
						Err(e) => {
							println!("Receive Loop: {:?}", e);
							return;
						}
					}
				}
				// Say what we received
				_ => {
                    match message {
                        OwnedMessage::Text(txt) => {
                            println!("{} {}", "<".blue(), txt.blue());
                        },
                        _ => println!("< {:?}", message)
                    }
                },
			}
		}
	});

    let mut rl = Editor::<()>::new()?;

    loop {
        print!("\r\u{001b}[2K\u{001b}[3D");
        let line: LineResult<String> = rl.readline(">> ");

        match line {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                rl.add_history_entry(line);
                // println!("Line: {}", line);

                let message = match line {
                    "/close" => {
                        // Close the connection
                        let _ = tx.send(OwnedMessage::Close(None));
                        break;
                    }
                    // Send a ping
                    "/ping" => OwnedMessage::Ping(b"PING".to_vec()),
                    // Otherwise, just send text
                    _ => OwnedMessage::Text(line.to_string()),
                };
        
                match tx.send(message) {
                    Ok(()) => (),
                    Err(e) => {
                        println!("Main Loop: {:?}", e);
                        break;
                    }
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }

    tx.send(OwnedMessage::Close(None)).expect("Unable to close connection.");

    // We're exiting
	println!("Waiting for child threads to exit");

	let _ = send_loop.join();
	let _ = receive_loop.join();

	println!("Exited");

    Ok(())
}

fn respond(line: &str) -> Result<bool, String> {
    let args = shlex::split(line).ok_or("error: Invalid quoting")?;
    let matches = cli()
        .try_get_matches_from(args)
        .map_err(|e| e.to_string())?;
    match matches.subcommand() {
        Some(("ping", _matches)) => {
            writeln!(std::io::stdout(), "Pong").map_err(|e| e.to_string())?;
            std::io::stdout().flush().map_err(|e| e.to_string())?;
        }
        Some(("quit", _matches)) => {
            writeln!(std::io::stdout(), "Exiting ...").map_err(|e| e.to_string())?;
            std::io::stdout().flush().map_err(|e| e.to_string())?;
            return Ok(true);
        }
        Some((name, _matches)) => unimplemented!("{}", name),
        None => unreachable!("subcommand required"),
    }

    Ok(false)
}

fn cli() -> Command {
    // strip out usage
    const PARSER_TEMPLATE: &str = "\
        {all-args}
    ";
    // strip out name/version
    const APPLET_TEMPLATE: &str = "\
        {about-with-newline}\n\
        {usage-heading}\n    {usage}\n\
        \n\
        {all-args}{after-help}\
    ";

    Command::new("repl")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_value_name("APPLET")
        .subcommand_help_heading("APPLETS")
        .help_template(PARSER_TEMPLATE)
        .subcommand(
            Command::new("ping")
                .about("Get a response")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("quit")
                .alias("exit")
                .about("Quit the REPL")
                .help_template(APPLET_TEMPLATE),
        )
}

fn readline() -> Result<String, String> {
    write!(std::io::stdout(), "> ").map_err(|e| e.to_string())?;
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| e.to_string())?;
    Ok(buffer)
}