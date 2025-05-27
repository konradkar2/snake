use bincode::config::Configuration as Bincode_cfg;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

pub struct Client {
    bincode_cfg: Bincode_cfg,
}
pub struct ClientHandle {
    handle: Arc<Client>,
}

impl ClientHandle {
    pub fn new(bincode_cfg: Bincode_cfg) -> Self {
        Self {
            handle: Arc::new(Client {
                bincode_cfg: bincode_cfg,
            }),
        }
    }
    pub fn launch_client(&self) {
        let handle = self.handle.clone();

        thread::spawn(move || {
            handle.run();
        });
    }
}

impl Client {
    fn run(&self) {}
}
