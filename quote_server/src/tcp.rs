//! Механизация TCP-сервера.

use crate::config::{WELCOME_INFO, WELCOME_SERVER};
use commons::errors::QuoteError;
use commons::traits::WriteExt;
use log::{error, info};
use macros::QuoteEnumDisplay;
use std::io::{BufRead, BufReader};
use std::net::TcpStream;
use std::str::FromStr;

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
pub fn handle_client(stream: TcpStream) {
    let mut writer = stream.try_clone().expect("Ошибка клонирования TcpStream");
    let mut reader = BufReader::new(&stream);

    writer.write_str(WELCOME_SERVER);
    writer.write_str(WELCOME_INFO);

    let ip_remote = get_addr_remote(&stream).unwrap_or_else(|| "н/д".to_string());

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => return,

            Ok(_) => {
                let input = line.trim();
                if input.is_empty() {
                    info!("Получена пустая строка от IP: {}", ip_remote);
                    continue;
                }

                info!("Получена команда: '{}' от IP: {}", input, ip_remote);
                match execute_command(&mut line) {
                    Ok(_) => {
                        writer.write_str("OK\n");
                        continue;
                    }
                    Err(e) => {
                        writer.write_str(format!("ОШИБКА: {}\n", e));
                        error!("Ошибка команды: {} от IP: {}", e, ip_remote);
                        continue;
                    }
                }
            }

            Err(_) => {
                error!("Ошибка чтения: '{}' от IP: {}", line.trim_end(), ip_remote);
                return;
            }
        }
    }
}

/// Обработка команд, полученных сервером.
fn execute_command(input: &mut str) -> Result<(), QuoteError> {
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
