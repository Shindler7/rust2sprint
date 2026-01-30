//! Quote Client. Приложение для взаимодействия с Quote Server.

use log::{info, warn};
// use std::io::{BufRead, BufReader, Result, Write};
use std::io::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

mod config;

use commons::init_simple_logger;
use commons::utils::get_workspace_root;
use config::LOG_FOLDER;

#[tokio::main]
async fn main() -> Result<()> {
    init_logger();

    info!("Quote Client запущен");
    let address = "127.0.0.1:8888";

    let stream = TcpStream::connect(&address).await?;
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    info!("Установлено соединение с сервером: {}", address);

    // Пропуск welcome-строк и технической информации.
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).await?;
        if bytes == 0 {
            break;
        }
        if line.trim_end().to_uppercase() == "READY" {
            break;
        }
    }

    let command = "STREAM udp://127.0.0.1:34254 ALL";

    // Отправка установочного запроса на сервер.
    writer.write_all(command.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    info!("Отправлена команда: {}", command);

    let mut server_response = String::new();
    let bytes = reader.read_line(&mut server_response).await?;
    if bytes == 0 {
        let err_msg = "Пустой ответ от сервера или сервер закрыл соединение.";
        warn!("{}", err_msg);
        panic!("{}", err_msg);
    }

    info!("Ответ сервера: {}", server_response.trim_end());

    Ok(())
}

/// Инициализировать логгер приложения.
///
/// Используется метод [`init_simple_logger`] из крейта [`commons`].
fn init_logger() {
    let log_folder = get_workspace_root().join(LOG_FOLDER);
    let app_name = env!("CARGO_PKG_NAME");
    init_simple_logger(app_name, log_folder);
}
