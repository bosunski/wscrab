use app::App;
use builder::Config;
use clap::{Parser, error::ErrorKind, CommandFactory};
use regex::Regex;
use futures::join;

mod builder;
mod app;

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
async fn main() {
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

    let config: Config = Config::builder().connect(url).build();
    let app: App = App::builder().configure(config).build();

    _ = join!(app.run());
}
