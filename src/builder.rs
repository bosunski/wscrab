#[derive(Debug, PartialEq)]
pub struct Config {
   pub auth: Option<String>,
   ca: Option<String>,
   cert: Option<String>,
   host: Option<String>,
   key: Option<String>,
   max_redirects: Option<usize>,
   no_color: Option<bool>,
   passphrase: Option<String>,
   proxy: Option<String>,
   slash: Option<bool>,
   pub connect: String,
   header: Option<String>,
   location: Option<bool>,
   listen: Option<u16>,
   no_check: Option<bool>,
   protocol: Option<String>,
   show_ping_pong: Option<bool>,
   subprotocol: Option<String>,
   wait: Option<String>,
   execute: Option<String>,
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
   auth: Option<String>,
   ca: Option<String>,
   cert: Option<String>,
   host: Option<String>,
   key: Option<String>,
   max_redirects: Option<usize>,
   no_color: Option<bool>,
   passphrase: Option<String>,
   proxy: Option<String>,
   slash: Option<bool>,
   connect: Option<String>,
   header: Option<String>,
   location: Option<bool>,
   listen: Option<u16>,
   no_check: Option<bool>,
   protocol: Option<String>,
   show_ping_pong: Option<bool>,
   subprotocol: Option<String>,
   wait: Option<String>,
   execute: Option<String>,
}

impl ConfigBuilder {
    // pub fn new(/* ... */) -> ConfigBuilder {
    //     ConfigBuilder {
    //         auth: None,
    //         ca: None,
    //         cert: None,
    //         host: None,
    //         key: None,
    //         max_redirects: None,
    //         no_color: None,
    //         passphrase: None,
    //         proxy: None,
    //         slash: None,
    //         connect: None,
    //         header: None,
    //         location: None,
    //         listen: None,
    //         no_check: None,
    //         protocol: None,
    //         show_ping_pong: None,
    //         subprotocol: None,
    //         wait: None,
    //         execute: None,
    //     }
    // }

    pub fn connect(mut self, url: String) -> ConfigBuilder {
        self.connect = Some(url);
        self
    }

    pub fn auth(mut self, auth: Option<String>) -> ConfigBuilder {
        self.auth = auth;
        self
    }

    pub fn build(self) -> Config {
        Config {
            auth: self.auth,
            ca: self.ca,
            cert: self.cert,
            host: self.host,
            key: self.key,
            max_redirects: self.max_redirects,
            no_color: self.no_color,
            passphrase: self.passphrase,
            proxy: self.proxy,
            slash: self.slash,
            connect: self.connect.expect("A URL is required for connection"),
            header: self.header,
            location: self.location,
            listen: self.listen,
            no_check: self.no_check,
            protocol: self.protocol,
            show_ping_pong: self.show_ping_pong,
            subprotocol: self.subprotocol,
            wait: self.wait,
            execute: self.execute,
        }
    }
}
