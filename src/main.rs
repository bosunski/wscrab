use rustyline_async::{Readline, ReadlineError, SharedWriter};

use std::{io::Write};

use futures::{prelude::*, join};
use futures::channel::mpsc::{self as futures_channel, UnboundedSender};

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use colored::*;
use clap::{Parser, error::ErrorKind, CommandFactory};
use regex::Regex;
use http::{Request, header};
use rand::rngs::OsRng;
use rand::RngCore;

/// Just another port of wscat to Rust
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Args {
   #[arg(long, value_name = "username:password", help = "add basic HTTP authentication header (--connect only)")]
   auth: Option<String>,

   #[arg(long, value_name = "ca", help = "specify a Certificate Authority (--connect only)")]
   ca: Option<String>,

   #[arg(long, value_name = "cert", help = "specify a Client SSL Certificate (--connect only)")]
   cert: Option<String>,

   #[arg(long, value_name = "host", help = "optional host")]
   host: Option<String>,

   #[arg(long, value_name = "key", help = "optional host")]
   key: Option<String>,

   #[arg(long, value_name = "num", default_value = "10", help = "maximum number of redirects allowed (--connect only) (default: 10)")]
   max_redirects: Option<usize>,

   #[arg(long, value_name = "", help = "run without color")]
   no_color: Option<bool>,

   #[arg(long, value_name = "passphrase", help = "specify a Client SSL Certificate Key's passphrase (--connect only). If you don't provide a value, it will be prompted for")]
   passphrase: Option<String>,

   #[arg(long, value_name = "[protocol://]host[:port]", help = "connect via a proxy. Proxy must support CONNECT method")]
   proxy: Option<String>,

   #[arg(long, value_name = "", help = "enable slash commands for control frames (/ping [data], /pong [data], /close [code [, reason]])")]
   slash: Option<bool>,

   #[arg(short, long, value_name = "url", help = "connect to a WebSocket server")]
   connect: Option<String>,

   #[arg(long, short = 'H', num_args = 1.., value_name = "header:value", help = "set an HTTP header. Repeat to set multiple (--connect only) (default: [])")]
   header: Option<String>,

   #[arg(long, short = 'L', value_name = "", help = "follow redirects (--connect only)")]
   location: Option<bool>,

   #[arg(long, short= 'l', value_name = "port", help = "listen on port")]
   listen: Option<u16>,

   #[arg(long, short = 'n', value_name = "", help = "do not check for unauthorized certificates")]
   no_check: Option<bool>,

   #[arg(long, short = 'p', value_name = "version", help = "optional protocol version")]
   protocol: Option<String>,

   #[arg(long, short = 'P', value_name = "", help = "print a notification when a ping or pong is received")]
   show_ping_pong: Option<bool>,

   #[arg(long, short = 's', value_name = "protocol", help = "optional subprotocol (default: [])")]
   subprotocol: Option<String>,

   #[arg(long, short = 'w', value_name = "seconds", help = "wait given seconds after executing command")]
   wait: Option<String>,

   #[arg(long, short = 'x', value_name = "command", help = "execute command after connecting")]
   execute: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), ReadlineError> {
    let args = Args::parse();
    let mut cmd = Args::command();

    let url = String::from(args.connect.as_deref().unwrap());
    let re = Regex::new(r"^\w+://.*$").unwrap();

    let mut modified_url = String::from("ws://");
    modified_url.push_str(&url);

    let url = if re.is_match(&url) {url} else {
        modified_url
    };

    if ! url.starts_with("ws") {
        cmd.error(ErrorKind::InvalidValue, "The URL must start with ws:// or wss://").exit();
    }

    let (rl, stdout) = Readline::new(">> ".to_owned()).unwrap();
    let (stdin_tx, stdin_rx) = futures_channel::unbounded();

    let request = create_request(url);

    let (ws_stream, _) = connect_async(request).await.expect("Failed to connect");
    writeln!(stdout.clone(), "{}", "Connected (press CTRL+C to quit)".green()).expect("TODO: panic message");

	let (writer, mut read) = ws_stream.split();
    let receiver_task = stdin_rx.map(Ok).forward(writer);
    let print_task = async {
        while let Some(message) = read.next().await {
            let mut stdout = stdout.clone();
            writeln!(stdout, "{} {}", "<<".blue(), message.unwrap().to_string().blue()).unwrap();
        }
    };

    _ = join!(print_task, receiver_task, read_line(stdin_tx, rl, stdout.clone()));

    Ok(())
}

fn create_request(url: String) -> Request<()> {
    let url = url::Url::parse(&url).unwrap();

   let mut key = [0u8; 16];
   OsRng.fill_bytes(&mut key);

   let key = base64::encode(key);

   // Set headers required for WebSocket handshake
   return Request::builder()
       .method("GET")
       .uri(url.as_str())
       .version(http::Version::HTTP_11)
       .header(header::CONNECTION, "Upgrade")
       .header(header::HOST, url.host_str().unwrap().to_owned())
       .header(header::UPGRADE, "websocket")
       .header(header::SEC_WEBSOCKET_VERSION, "13")
       .header(header::SEC_WEBSOCKET_KEY, key)
       .body(())
       .unwrap();
}

async fn read_line(tx: UnboundedSender<Message>, mut rl: Readline, mut stdout: SharedWriter) -> Result<(), ReadlineError> {
    loop {
        futures::select! {
			command = rl.readline().fuse() => match command {
				Ok(line) => {
					let line = line.trim();
					rl.add_history_entry(line.to_owned());
                    match line {
						_ => {
							tx.unbounded_send(Message::Text(line.to_string())).unwrap();
						},
					}
				},
				Err(ReadlineError::Eof) => { writeln!(stdout, "Exiting...")?; break },
				Err(ReadlineError::Interrupted) => { writeln!(stdout, "^C")?; break },
				// Err(ReadlineError::Closed) => break, // Readline was closed via one way or another, cleanup other futures here and break out of the loop
				Err(err) => {
					writeln!(stdout, "Received err: {:?}", err)?;
					writeln!(stdout, "Exiting...")?;
					break
				},
			}
		}
    }

	rl.flush()?;
    Ok(())
}
