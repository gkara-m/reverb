use std::net::Ipv6Addr;

use crate::CONFIG;



// If this file is edited ever the version should be updated
static VERSION: &str = "0.1.0";


struct Internet {
    name: String,
    connected_to: Vec<Connection>, 
}

struct ConnectionOpen {
    version: String,
    name: String,
    capabilities: Vec<String>,
}

struct Connection {
    name: String,
    capabilities: Vec<String>,
    address: Ipv6Addr,
}

impl Internet {
    pub fn new() -> Internet {
        Internet {
            name: (&CONFIG.get().unwrap().device_name).clone(),
            connected_to: Vec::new(),
        }
    }

    fn create_connection_open(&self) -> ConnectionOpen {
        ConnectionOpen { 
            version: VERSION.to_string(), 
            name: self.name.clone(), 
            capabilities: vec!["local".to_string()], //TODO: change to use the config (will also need to change config)
        }
    }

    pub fn open_new_connection(address: Ipv6Addr) {
        
    }
}