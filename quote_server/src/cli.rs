//! Обработка аргументов командной строки при запуске приложения.
//! ## Пример
//!
//! ```
//! $ qserver --port 8888
//! ```

use crate::config::{DEFAULT_SERVER_PORT, SERVER_ADDRESS, TCP_PORTS_ALLOWED};
use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[clap(about = "Quote Server. Generating and broadcasting real-time ticker quotes.")]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    /// TCP port to listen on (server binds to 127.0.0.1:PORT).
    #[clap(short, long, required = false, default_value_t = DEFAULT_SERVER_PORT, value_parser=port_in_range)]
    port: u16,
}

/// Валидатор для поля `port`.
fn port_in_range(s: &str) -> Result<u16, String> {
    let port: usize = s.parse().map_err(|_| format!("invalid port number: {s}"))?;
    if TCP_PORTS_ALLOWED.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port number {} not in range {} — {}",
            s,
            TCP_PORTS_ALLOWED.start(),
            TCP_PORTS_ALLOWED.end()
        ))
    }
}

/// Параметры, полученные из командной строки при запуске приложения.
///
/// ## Доступные данные
/// - `server_addr` — сформированный экземпляр [`SocketAddr`] с адресом сокета
///   сервера и портом. Например, `127.0.0.1:8888`.
#[derive(Debug)]
pub struct ServerSet {
    /// Адрес работы TCP-сервера.
    pub server_addr: SocketAddr,
}

impl ServerSet {
    /// Создать экземпляр на основе аргументов из командной строки.
    fn new(args: &CliArgs) -> Self {
        let server_addr = Self::get_server_addr(args.port);

        Self { server_addr }
    }

    /// Предоставить адрес TCP-сервера.
    fn get_server_addr(port: u16) -> SocketAddr {
        SocketAddr::from((SERVER_ADDRESS, port))
    }
}

/// Получить от пользователя первичные настройки приложения.
pub fn parse_cli_args() -> ServerSet {
    let args = CliArgs::parse();

    ServerSet::new(&args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn port_validator_accepts_allowed_port() {
        let ok_port = TCP_PORTS_ALLOWED.start().to_string();
        let res = port_in_range(&ok_port);
        assert!(res.is_ok());
    }

    #[test]
    fn port_validator_rejects_out_of_range() {
        let bad_port = (TCP_PORTS_ALLOWED.end() + 1).to_string();
        let res = port_in_range(&bad_port);
        assert!(res.is_err());
    }

    #[test]
    fn server_set_builds_correct_addr() {
        let port = DEFAULT_SERVER_PORT;
        let args = CliArgs::parse_from(["qserver", "--port", &port.to_string()]);
        let set = ServerSet::new(&args);

        assert_eq!(set.server_addr, SocketAddr::from((SERVER_ADDRESS, port)));
    }
}
