//! Конфигурационный файл Quote Client.

use std::net::Ipv4Addr;
use std::ops::RangeInclusive;
use std::str::FromStr;

/// Название директории для log-файлов.
pub const LOG_FOLDER: &str = "log";

/// Адрес TCP-сервера по умолчанию.
const DEFAULT_SERVER_SOCKET: &str = "127.0.0.1";

pub fn default_server_socket() -> Ipv4Addr {
    Ipv4Addr::from_str(DEFAULT_SERVER_SOCKET)
        .unwrap_or_else(|e| panic!("Invalid default server address: {}", e))
}

/// Порт для подключения к TCP-серверу по умолчанию.
pub const DEFAULT_SERVER_PORT: u16 = 8888;

/// Диапазон разрешённых TCP-портов.
pub const ALLOW_TCP_PORTS: RangeInclusive<u16> = RangeInclusive::new(1024, 49151);

/// Диапазон разрешённых в приложении UDP-портов.
pub const ALLOW_UDP_PORTS: RangeInclusive<u16> = RangeInclusive::new(1024, 49151);

/// Базовый UDP-адрес для приёма данных от сервера.
pub const UDP_CALLBACK: &str = "127.0.0.1";
