


pub trait DNSServer {
    fn run_server(self);

    fn shutdown(&self);
}

