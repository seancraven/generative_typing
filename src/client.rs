// Current state, can send and recive messages from the server.
//
use std::{
    io::{self, prelude::*},
    net::TcpStream,
};
#[derive(Debug, Clone)]
pub struct IPV4 {
    address: String,
}
impl IPV4 {
    fn address(&self, port: String) -> String {
        format!("{}:{}", self.address, port)
    }
    fn new(address: String) -> IPV4 {
        IPV4 { address }
    }
}
pub struct TypeClient {
    host_ip: IPV4,
    port: String,
}
impl TypeClient {
    pub fn new(host_ip: IPV4, port: String) -> TypeClient {
        TypeClient { host_ip, port }
    }
    pub fn address(&self) -> String {
        self.host_ip.address(self.port.clone())
    }
    /// Client starts, forms a connection and then returns an iterator over the response.
    /// The current design doesn't make sense that you return a stream and leak this.
    pub fn start_gen(&self) -> Result<TcpStream, io::Error> {
        let address = self.address();
        println!("Connecting to {}", address);
        let mut stream = TcpStream::connect(&address)?;
        stream.write_all("if __name__ == '__main__':".as_bytes())?;
        return Ok(stream);
    }
    pub fn new_from_env() -> Result<TypeClient, std::env::VarError> {
        let ip = IPV4::new(std::env::var("HOST")?);
        let port = std::env::var("PORT")?;
        Ok(TypeClient::new(ip, port))
    }
}

#[cfg(test)]
mod client_test {
    use super::*;
    use dotenv::dotenv;
    use std::{
        env::var,
        io::{BufRead, BufReader},
    };
    #[test]
    fn server_test() {
        dotenv().ok();
        let ip = IPV4::new(var("IPV4").expect("Can't find .evn variable IPV4"));
        let port = var("PORT").expect("Can't find .evn variable IPV4");
        let client = TypeClient::new(ip, port);
        let stream = client.start_gen().expect("Failed to connect to host");
        let reader = BufReader::new(stream);
        reader
            .lines()
            .map(|line| line.expect("Failed to read line"))
            .enumerate()
            .for_each(|(idx, line)| {
                println!("Iteration: {}", idx);
                println!("{}", line);
            });
    }
}
