use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use bincode::config::Configuration as Bincode_cfg;

pub struct Server
{
    bincode_cfg: Bincode_cfg,
}
pub struct ServerHandle {
    handle: Arc<Server>,
}

use crate::ifc::Message;

impl ServerHandle {
    pub fn new(bincode_cfg: Bincode_cfg) -> Self {
        Self {
            handle: Arc::new(Server { bincode_cfg: bincode_cfg}),
        }
    }
    pub fn launch_server(&self) {
        let handle = self.handle.clone();

        thread::spawn(move || {
            handle.server();
        });
    }
}

impl Server {
    fn server(&self) {
        let listener = TcpListener::bind("127.0.0.1:223").expect("binds succesfully");
        println!("[SERVER]: Listening for connection...");
        // accept connections and process them serially
        for stream in listener.incoming() {
            self.handle_client(stream.expect("valid client "));
        }
    }

    fn handle_client(&self, mut stream: TcpStream) {
    
    let mut buffer = [0; 4096];
    let read_n_bytes = stream.read(&mut buffer).unwrap_or_default();

    println!("[SERVER]: read bytes: {}", read_n_bytes);

    let (decoded, _): (Message, usize) =
        bincode::decode_from_slice(&buffer[..read_n_bytes], self.bincode_cfg).unwrap();

    println!("[SERVER]: got msg: {:?}", decoded);
}

}
