//! Обработка аргументов командной строки при запуске приложения.
//!
//! Пользователь может указать:
//! - адрес и порт TCP-сервера
//! - порт для приёма UDP-данных
//! - путь к файлу со списком тикеров для подписки

use crate::config::*;
use clap::{Parser, Subcommand};
use commons::get_ticker_data;
use log::error;
use std::fmt::{Display, Formatter};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::process::exit;
use url::Url;

/// Перечисление ошибок при завершении приложения.
#[derive(Copy, Clone)]
#[repr(u8)]
#[allow(dead_code)]
enum ExitCode {
    /// Ошибка формирования сокета TCP.
    InvalidServerSocket = 1,
    /// Ошибка формирования ссылки UDP.
    InvalidUDP,
}

impl ExitCode {
    /// Предоставить цифровое значение выбранного перечисления (`u8`).
    ///
    /// ## Пример
    ///
    /// ```
    /// use cli::ExitCode;
    ///
    /// let code = ExitCode::InvalidUDP;
    /// assert_eq!(code.value(), 2u8);
    /// ```  
    pub fn value(&self) -> u8 {
        *self as u8
    }
}

#[derive(Debug, Parser)]
#[command(about = "Quote Client. Real-time ticker data streaming.")]
#[command(author, version, long_about = None)]
struct CliArgs {
    /// TCP server socket address.
    #[arg(short, long, required = false, default_value_t = default_server_socket())]
    socket: Ipv4Addr,

    /// TCP server port (for example 8888).
    #[arg(short, long, required = false, default_value_t = DEFAULT_SERVER_PORT, value_parser=validate_tcp_port
    )]
    port: u16,

    /// UDP port for receiving data (for example 34254).
    #[arg(short, long, required = true, value_parser=validate_udp_port)]
    udp: u16,

    /// Supported server commands.
    #[command(subcommand)]
    command: Commands,
}

/// Валидатор для полей `port` и аналогичных.
fn port_in_range(s: &str, range: RangeInclusive<u16>) -> Result<u16, String> {
    let port: u16 = s.parse().map_err(|_| format!("invalid port number: {s}"))?;
    if range.contains(&port) {
        Ok(port)
    } else {
        Err(format!(
            "port number {} not in range {} — {}",
            s,
            range.start(),
            range.end()
        ))
    }
}

/// Валидатор для поля `port`.
fn validate_tcp_port(s: &str) -> Result<u16, String> {
    port_in_range(s, ALLOW_TCP_PORTS)
}

/// Валидатор для поля `udp`.
fn validate_udp_port(s: &str) -> Result<u16, String> {
    port_in_range(s, ALLOW_UDP_PORTS)
}

/// Supported server commands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Fetches data for tickers. `File path`: filters to tickers listed in the
    /// file. `No file`: returns all ticker data (ALL)
    Stream {
        #[arg(short, long, required = false, value_name = "FILE")]
        file: Option<PathBuf>,
    },
    /// Cancel previously scheduled data transmission.
    Cancel,
}

/// Параметры, полученные из командной строки при запуске приложения.
pub struct ClientSet {
    /// Адрес TCP-сервера.
    pub server_addr: SocketAddr,
    /// UDP-адрес для получения данных.
    pub udp_url: Url,
    /// Список тикеров для подписки.
    #[allow(dead_code)]
    pub tickers: Vec<String>,
    /// Подготовленная команда для сервера.
    pub command: String,
}

impl Display for ClientSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "server: {} | udp: {}", self.server_addr, self.udp_url)
    }
}

impl ClientSet {
    /// Сформировать экземпляр [`ClientSet`] на основе данных из командной
    /// строки.
    ///
    /// При обнаружении ошибок в значениях приложение завершиться.
    fn new(args: &CliArgs) -> Self {
        let server_addr = Self::make_server_addr(args.socket, args.port);
        let udp_url = Self::make_udp_url(args.udp);
        let (tickers, command) = Self::tickers_and_command(&args.command, &udp_url);

        Self {
            server_addr,
            udp_url,
            tickers,
            command,
        }
    }

    /// Сформировать адрес сокета TCP-сервера.
    ///
    /// ## Args
    ///
    /// - `address` — валидный сокет в [`Ipv4Addr`]
    /// - `port` — корректный TCP-порт, в разрешённом конфигурацией приложения
    ///   диапазоне
    fn make_server_addr(address: Ipv4Addr, port: u16) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(address, port))
    }

    /// Проверить UDP-порт и вернуть корректный UDP-адрес.
    ///
    /// В случае ошибки приложение завершается с выводом причины.
    fn make_udp_url(port_udp: u16) -> Url {
        Url::parse(&format!("udp://{}:{}", UDP_CALLBACK, port_udp)).unwrap_or_else(|error| {
            let err_msg = format!(
                "не удалось сформировать `udp_url` (base_url: {}, port: {}): {}",
                UDP_CALLBACK, port_udp, error
            );
            exit_err(&err_msg, ExitCode::InvalidUDP)
        })
    }

    /// Получить список тикеров для подписки из файла по переданной ссылке.
    fn get_tickers(path: &PathBuf) -> Vec<String> {
        get_ticker_data(path).unwrap_or_default()
    }

    /// Сформировать команду для сервера на основе пользовательского выбора,
    /// а также вернуть список отобранных тикеров, когда это требуется.
    fn tickers_and_command(command: &Commands, udp_url: &Url) -> (Vec<String>, String) {
        match command {
            Commands::Stream { file: t_file } => {
                let tickers = match t_file {
                    Some(t) => Self::get_tickers(t),
                    None => {
                        vec![]
                    }
                };

                let args = match tickers.is_empty() {
                    true => "ALL".to_string(),
                    false => tickers.join(","),
                };

                (tickers, format!("STREAM {} {}", udp_url, args))
            }
            Commands::Cancel => (vec![], format!("CANCEL {}", udp_url)),
        }
    }
}

/// Получить от пользователя первичные настройки приложения.
///
/// Гарантировано, что данные получены и проверены в доступных пределах.
/// Например, что `server_addr` содержит ссылку и порт (но не гарантируется,
/// что ссылка ведёт к действующему серверу).
///
/// ## Обработка ошибок
///
/// Если полученные данные некорректные, приложение завершает работу с выводом
/// сообщения об ошибке в консоль и log-файл. При завершении работы приложение
/// возвращает ОС ошибку, в соответствии с [`ExitCode`].
pub fn parse_cli_args() -> ClientSet {
    let args = CliArgs::parse();

    ClientSet::new(&args)
}

/// Опубликовать сообщение об ошибке и завершить работу приложения.
fn exit_err(message: &str, code: ExitCode) -> ! {
    error!("Ошибка: {} (код {})", message, code.value());
    eprintln!("Ошибка: {}", message);
    exit(code.value() as i32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn tcp_port_validator_accepts_allowed() {
        let ok = ALLOW_TCP_PORTS.start().to_string();
        assert!(validate_tcp_port(&ok).is_ok());
    }

    #[test]
    fn udp_port_validator_rejects_out_of_range() {
        let bad = (ALLOW_UDP_PORTS.end() + 1).to_string();
        assert!(validate_udp_port(&bad).is_err());
    }

    #[test]
    fn make_udp_url_is_correct() {
        let url = ClientSet::make_udp_url(34254);
        assert_eq!(url.as_str(), format!("udp://{}:34254", UDP_CALLBACK));
    }

    #[test]
    fn stream_command_all_if_no_file() {
        let udp_url = Url::parse("udp://127.0.0.1:34254").unwrap();
        let (tickers, cmd) =
            ClientSet::tickers_and_command(&Commands::Stream { file: None }, &udp_url);

        assert!(tickers.is_empty());
        assert_eq!(cmd, "STREAM udp://127.0.0.1:34254 ALL");
    }

    #[test]
    fn stream_command_from_file() {
        let tmp = std::env::temp_dir().join("tickers_test.txt");
        fs::write(&tmp, "AAPL\nTSLA\n").unwrap();

        let udp_url = Url::parse("udp://127.0.0.1:34254").unwrap();
        let (tickers, cmd) =
            ClientSet::tickers_and_command(&Commands::Stream { file: Some(tmp) }, &udp_url);

        assert_eq!(tickers, vec!["AAPL", "TSLA"]);
        assert_eq!(cmd, "STREAM udp://127.0.0.1:34254 AAPL,TSLA");
    }
}
