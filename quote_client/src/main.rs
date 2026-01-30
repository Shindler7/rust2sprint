//! Quote Client. Приложение для взаимодействия с Quote Server.

use std::net::TcpStream;

mod config;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
}
