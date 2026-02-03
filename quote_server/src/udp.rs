//! Механизация серверного UDP-протокола.

use crate::config::{CHANNEL_TIMEOUT_MS, SOCKET_READ_TIMEOUT_MS, UDP_PING_TIMEOUT_SECS};
use crate::models::ClientSubscription;
use commons::models::StockQuote;
use log::{error, info, warn};
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
            .set_read_timeout(Some(Duration::from_millis(SOCKET_READ_TIMEOUT_MS)))
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

            if let Ok(quote) = client
                .recv
                .recv_timeout(Duration::from_millis(CHANNEL_TIMEOUT_MS))
            {
                let stock_quote: StockQuote = match serde_json::from_str(&quote) {
                    Ok(q) => q,
                    Err(e) => {
                        warn!("Некорректная строка от генератора: {quote} — {e}");
                        return;
                    }
                };

                if !client.tickers.is_empty() && !client.tickers.contains(&stock_quote.ticker) {
                    return;
                }

                let _ = socket.send_to(quote.as_bytes(), udp_addr);
            }
        }

        info!("UDP трансляция остановлена");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use commons::models::{StockQuote, Transaction};
    use crossbeam_channel::unbounded;
    use std::collections::HashSet;
    use std::net::{SocketAddr, UdpSocket};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Duration;
    use url::Url;

    fn make_client(
        udp_addr: SocketAddr,
        tickers: HashSet<String>,
        sender: crossbeam_channel::Sender<String>,
        recv: crossbeam_channel::Receiver<String>,
        stop: Arc<AtomicBool>,
    ) -> ClientSubscription {
        ClientSubscription {
            unique_id: 1,
            tcp_addr: "127.0.0.1:1".parse().unwrap(),
            udp_url: Url::parse(&format!("udp://{}", udp_addr)).unwrap(),
            tickers,
            sender,
            recv,
            stop_flag: stop,
        }
    }

    fn sample_quote(ticker: &str) -> StockQuote {
        StockQuote {
            ticker: ticker.to_string(),
            price: 100.0,
            volume: 1000,
            transaction: Transaction::Buy,
            timestamp: 1,
        }
    }

    #[test]
    fn stream_sends_json_when_all() {
        let recv_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        recv_socket
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let udp_addr = recv_socket.local_addr().unwrap();

        let (tx, rx) = unbounded();
        let stop = Arc::new(AtomicBool::new(false));
        let client = make_client(udp_addr, HashSet::new(), tx.clone(), rx, stop.clone());

        spawn_stream(client);

        let quote = sample_quote("AAPL");
        let quote_json = serde_json::to_string(&quote).unwrap();
        tx.send(quote_json).unwrap();

        let mut buf = [0u8; 1024];
        let (size, _) = recv_socket.recv_from(&mut buf).unwrap();
        let json = std::str::from_utf8(&buf[..size]).unwrap();
        let parsed: StockQuote = serde_json::from_str(json).unwrap();

        assert_eq!(parsed.ticker, "AAPL");

        stop.store(true, Ordering::SeqCst);
    }

    #[test]
    fn stream_filters_unmatched_ticker() {
        let recv_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        recv_socket
            .set_read_timeout(Some(Duration::from_millis(300)))
            .unwrap();
        let udp_addr = recv_socket.local_addr().unwrap();

        let (tx, rx) = unbounded();
        let stop = Arc::new(AtomicBool::new(false));

        let mut tickers = HashSet::new();
        tickers.insert("AAPL".to_string());

        let client = make_client(udp_addr, tickers, tx.clone(), rx, stop.clone());

        spawn_stream(client);

        let quote = sample_quote("MSFT");
        let quote_json = serde_json::to_string(&quote).unwrap();
        tx.send(quote_json).unwrap();

        let mut buf = [0u8; 128];
        let res = recv_socket.recv_from(&mut buf);

        assert!(res.is_err());

        stop.store(true, Ordering::SeqCst);
    }
}
