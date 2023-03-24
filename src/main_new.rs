use rustyline_async::{Readline, ReadlineError, SharedWriter};

use std::{io::Write, time::Duration};

use futures::{prelude::*, join};
use futures::channel::mpsc::{self as futures_channel, UnboundedSender};
use tokio::time::{self};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

const CONNECTION: &'static str = "ws://127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), ReadlineError> {
    let (rl, stdout) = Readline::new(">> ".to_owned()).unwrap();
    let (stdin_tx, stdin_rx) = futures_channel::unbounded();

    simplelog::WriteLogger::init(
        log::LevelFilter::Debug,
        simplelog::Config::default(),
        stdout.clone(),
    )
        .unwrap();

    let url = url::Url::parse(CONNECTION).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");
    let (writer, mut read) = ws_stream.split();

    let receiver_task = stdin_rx.map(Ok).forward(writer);
    let print_task = async {
        while let Some(message) = read.next().await {
            let mut stdout = stdout.clone();
            writeln!(stdout, "<< {}", message.unwrap()).unwrap();
        }
    };

    join!(print_task, receiver_task, read_line(stdin_tx, rl, stdout.clone()));

    // Flush all writers to stdout
    // rl.flush()?;

    Ok(())
}

async fn read_line(tx: UnboundedSender<Message>, mut rl: Readline, mut stdout: SharedWriter) -> Result<(), ReadlineError> {
    let mut periodic_timer1 = time::interval(Duration::from_secs(2));
    let mut periodic_timer2 = time::interval(Duration::from_secs(3));

    let mut running_first = true;
    let mut running_second = false;

    loop {
        futures::select! {
			_ = periodic_timer1.tick().fuse() => {
				if running_first { writeln!(stdout, "First timer went off!")?; }
			}
			_ = periodic_timer2.tick().fuse() => {
				if running_second { log::info!("Second timer went off!"); }
			}
			command = rl.readline().fuse() => match command {
				Ok(line) => {
					let line = line.trim();
					rl.add_history_entry(line.to_owned());
					tx.unbounded_send(Message::Text(line.to_string())).unwrap();

                    match line {
						"start task" => {
							writeln!(stdout, "Starting the task...")?;
							running_first = true;
						},
						"stop task" => {
							writeln!(stdout, "Stopping the task...")?;
							running_first = false;
						}
						"start logging" => {
							log::info!("Starting the logger...");
							running_second = true
						},
						"stop logging" => {
							log::info!("Stopping the logger...");
							running_second = false
						},
						"info" => {
							writeln!(stdout, r"
hello there
I use NixOS btw
its pretty cool
							")?;
						}
						_ => writeln!(stdout, "Command not found: \"{}\"", line)?,
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

    Ok(())
}
