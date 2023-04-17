use crate::builder::Config;
use rustyline_async::{Readline, ReadlineError, SharedWriter};

use std::io::Write;

use futures::{prelude::*, join};
use futures::channel::mpsc::{self as futures_channel, UnboundedSender};

use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use colored::*;
use http::{Request, header};
use rand::rngs::OsRng;
use rand::RngCore;


#[derive(Debug, PartialEq)]
pub struct App {
    pub config: Config,
}

#[derive(Default)]
pub struct AppBuilder {
    config: Option<Config>
}

impl App {
    pub fn builder() -> AppBuilder {
        AppBuilder::default()
    }

    pub async fn run(&self) -> Result<(), ReadlineError> {
        let (rl, stdout) = Readline::new(">> ".to_owned()).unwrap();
        let (stdin_tx, stdin_rx) = futures_channel::unbounded();

        let request = self.create_request(&self.config.connect);
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

        _ = join!(print_task, receiver_task, self.read_line(stdin_tx, rl, stdout.clone()));

        Ok(())
    }

    fn create_request(&self, url: &String) -> Request<()> {
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
           .header(header::HOST, url.host_str().unwrap().clone())
           .header(header::UPGRADE, "websocket")
           .header(header::SEC_WEBSOCKET_VERSION, "13")
           .header(header::SEC_WEBSOCKET_KEY, key)
           .body(())
           .unwrap();
    }

    async fn read_line(&self, tx: UnboundedSender<Message>, mut rl: Readline, mut stdout: SharedWriter) -> Result<(), ReadlineError> {
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
}

impl AppBuilder {
    pub fn new() -> AppBuilder {
        AppBuilder { config: None }
    }

    pub fn configure(mut self, config: Config) -> AppBuilder {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> App {
        App { config: self.config.expect("Config must be set to create app instance") }
    }
}

