//! Механизация TCP-сервера.

use crate::config::{WELCOME_INFO, WELCOME_SERVER};
use commons::errors::QuoteError;
use log::{error, info};
use macros::QuoteEnumDisplay;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// Перечисление возможных команд для сервера.
#[derive(QuoteEnumDisplay)]
enum Commands {
    /// Запрос на получение информации о тикерах.
    #[str("stream")]
    Stream,
}

impl Commands {
    /// Диспетчер исполнения команд к серверу.
    ///
    /// На основании переданной команды вызывает соответствующий метод для
    /// обработки.
    pub fn execute(&self, args: Vec<String>) {
        match self {
            Commands::Stream => Self::stream(args),
        }
    }

    /// Обработчик команды `STREAM`.
    ///
    /// ## STREAM
    ///
    /// Получение данных о тикерах. Допустимые команды:
    /// - `STREAM <URL>:<PORT> ALL`
    /// - `STREAM <URL>:<PORT> <TICKERS, ...>`
    fn stream(args: Vec<String>) {}
}

/// Обработчик входных данных соединения.
pub async fn handle_client(stream: TcpStream) -> std::io::Result<()> {
    let ip_remote = get_addr_remote(&stream).unwrap_or_else(|| "н/д".to_string());

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    writer.write_all(WELCOME_SERVER.as_bytes()).await?;
    writer.write_all(WELCOME_INFO.as_bytes()).await?;
    writer.flush().await?;

    // Терминатор.
    writer.write_all(b"READY\n").await?;

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => return Ok(()),

            Ok(_) => {
                let input = line.trim();
                if input.is_empty() {
                    info!("Получена пустая строка от IP: {}", ip_remote);
                    continue;
                }

                info!("Получена команда: '{}' от IP: {}", input, ip_remote);
                match execute_command(input) {
                    Ok(_) => {
                        writer.write_all(b"OK\n").await?;
                    }
                    Err(e) => {
                        writer
                            .write_all(format!("ОШИБКА: {}\n", e).as_bytes())
                            .await?;
                        error!("Ошибка команды: {} от IP: {}", e, ip_remote);
                    }
                }
            }

            Err(_) => {
                error!("Ошибка чтения: '{}' от IP: {}", line.trim_end(), ip_remote);
                return Ok(());
            }
        }
    }
}

/// Обработка команд, полученных сервером.
fn execute_command(input: &str) -> Result<(), QuoteError> {
    let mut parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();

    let command = match Commands::from_str(parts.remove(0).as_str()) {
        Ok(command) => command,
        Err(e) => return Err(QuoteError::command_err(e.to_string())),
    };

    command.execute(parts);

    Ok(())
}

/// Вернуть IP-адрес клиента (и порт), подключившегося к серверу.
///
/// Если определить адрес не удалось, возвращает `None`.
fn get_addr_remote(stream: &TcpStream) -> Option<String> {
    match stream.peer_addr() {
        Ok(a) => Some(a.to_string()),
        Err(_) => None,
    }
}
