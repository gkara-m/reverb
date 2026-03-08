

static version: &str = "0.1";


struct Internet {
    name: String,
    connected_to: Vec<Connection>, 
}

struct Connection {
    version: String,
    name: String,
    capabilities: Vec<String>,
}