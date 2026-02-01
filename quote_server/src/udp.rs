//! Механизация серверного UDP-протокола.

use crate::config::UDP_PING_TIMEOUT_SECS;
use crate::tcp::ClientSubscription;
use log::{error, info};
use std::{
    net::UdpSocket,
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

/// Запустить UDP-поток для клиента.
pub fn spawn_stream(client: ClientSubscription) {
    thread::spawn(move || {
        let udp_addr = client
            .udp_url
            .socket_addrs(|| None)
            .ok()
            .and_then(|v| v.first().cloned());

        let Some(udp_addr) = udp_addr else {
            error!("Некорректный UDP адрес");
            return;
        };

        let socket = UdpSocket::bind("0.0.0.0:0").expect("Не удалось привязаться к UDP-сокету");
        socket
            .set_read_timeout(Some(Duration::from_millis(500)))
            .expect("Ошибка параметра `set_read_timeout`");

        info!("UDP трансляция на адрес: {}", udp_addr);

        let mut last_ping = Instant::now();
        let mut buf = [0u8; 64];

        loop {
            if client.stop_flag.load(Ordering::SeqCst) {
                break;
            }

            if last_ping.elapsed() > Duration::from_secs(UDP_PING_TIMEOUT_SECS) {
                info!("Таймаут ожидания пинга от клиента. Трансляция прервана");
                break;
            }

            if let Ok((size, _)) = socket.recv_from(&mut buf) {
                let msg = String::from_utf8_lossy(&buf[..size]).to_ascii_lowercase();
                if msg.trim() == "ping" {
                    last_ping = Instant::now();
                }
            }

            if let Ok(quote) = client.recv.recv_timeout(Duration::from_millis(200))
                && let Some(ticker) = extract_ticker(&quote)
                && client.tickers.contains(ticker)
            {
                let _ = socket.send_to(quote.as_bytes(), udp_addr);
            }
        }

        info!("UDP трансляция остановлена");
    });
}

fn extract_ticker(quote: &str) -> Option<&str> {
    quote
        .split([',', '|', ' ', '\t'])
        .next()
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::extract_ticker;

    #[test]
    fn extract_ticker_pipe_format() {
        let quote = "AAPL|123.4|100|123456|buy";
        assert_eq!(extract_ticker(quote), Some("AAPL"));
    }

    #[test]
    fn extract_ticker_comma_format() {
        let quote = "TSLA,123.4,100,123456,buy";
        assert_eq!(extract_ticker(quote), Some("TSLA"));
    }

    #[test]
    fn extract_ticker_empty() {
        let quote = "   ";
        assert_eq!(extract_ticker(quote), None);
    }
}
