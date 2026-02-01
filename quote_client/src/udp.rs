//! UDP-клиент для приёма котировок и отправки Ping.

use log::info;
use std::{
    io,
    net::{SocketAddr, UdpSocket},
    sync::{
        atomic::{AtomicBool, Ordering}, Arc,
        Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use url::Url;

/// UDP-клиент.
pub struct UdpClient {
    socket: UdpSocket,
    server_addr: Arc<Mutex<Option<SocketAddr>>>,
}

impl UdpClient {
    /// Создать UDP-сокет для приёма котировок по ссылке.
    pub fn bind_url(url: &Url) -> io::Result<Self> {
        let addr = url.socket_addrs(|| None)?.first().cloned().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "Некорректная UDP-ссылка")
        })?;

        Self::bind(addr)
    }

    /// Создать UDP-сокет для приёма котировок (по адресу сокета).
    pub fn bind(addr: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(Some(Duration::from_millis(500)))?;
        Ok(Self {
            socket,
            server_addr: Arc::new(Mutex::new(None)),
        })
    }

    /// Запустить поток Ping.
    pub fn spawn_ping(&self, stop: Arc<AtomicBool>) -> JoinHandle<()> {
        let socket = self
            .socket
            .try_clone()
            .expect("Не удалось клонировать 'socket' UDP");
        let addr = Arc::clone(&self.server_addr);

        thread::spawn(move || {
            let mut last = Instant::now();

            // Ping ping ping.
            loop {
                if stop.load(Ordering::SeqCst) {
                    break;
                }

                if last.elapsed() >= Duration::from_secs(2) {
                    if let Some(target) = *addr.lock().unwrap() {
                        let _ = socket.send_to(b"Ping", target);
                    }
                    last = Instant::now();
                }

                thread::sleep(Duration::from_millis(100));
            }
        })
    }

    /// Основной цикл приёма котировок.
    pub fn recv_loop(&self, stop: Arc<AtomicBool>) {
        let mut buf = [0u8; 1024];

        loop {
            if stop.load(Ordering::SeqCst) {
                break;
            }

            match self.socket.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    self.set_server_addr(addr);
                    let msg = String::from_utf8_lossy(&buf[..size]);
                    println!("{}", msg.trim_end());
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(_) => break,
            }
        }

        info!("UDP-приёмник остановлен");
    }

    fn set_server_addr(&self, addr: SocketAddr) {
        let mut guard = self.server_addr.lock().unwrap();
        if guard.is_none() {
            *guard = Some(addr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    #[test]
    fn bind_url_parses_valid_udp() {
        let url = Url::parse("udp://127.0.0.1:34567").unwrap();
        let client = UdpClient::bind_url(&url);
        assert!(client.is_ok());
    }

    #[test]
    fn server_addr_set_only_once() {
        let url = Url::parse("udp://127.0.0.1:0").unwrap();
        let client = UdpClient::bind_url(&url).unwrap();

        let addr1 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1111));
        let addr2 = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 2222));

        client.set_server_addr(addr1);
        client.set_server_addr(addr2);

        let stored = client.server_addr.lock().unwrap().unwrap();
        assert_eq!(stored, addr1);
    }
}
