//! Quote Server. Консольное приложение генерации котировок о ценах акций.
//! Например, для тикеров "AAPL", "GOOGL", "TSLA". Данные включают ряд
//! параметров, которые можно дополнять.

#![warn(missing_docs)]

use commons::utils::get_workspace_root;
use log::{error, info};
use tokio::net::TcpListener;

mod config;
mod generator;
mod tcp;
mod udp;

use commons::init_simple_logger;
use config::{LOG_FOLDER, server_endpoint};
use tcp::handle_client;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Инициализация логгера.
    init_logger();

    let endpoint = server_endpoint();
    let listener = TcpListener::bind(&endpoint).await?;
    println!("Запущен сервер по адресу {}", &endpoint);
    println!("Завершить работу сервера с помощью CTRL-C/CTRL-BREAK.\n");

    info!("Quote Server запущен");

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                info!("Рукопожатие: {:?}", addr);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream).await {
                        error!("Ошибка обработки клиента: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Ошибка работы сервера: {}", e);
            }
        }
    }
}

/// Инициализировать логгер приложения.
///
/// Используется метод [`init_simple_logger`] из крейта [`commons`].
fn init_logger() {
    let log_folder = get_workspace_root().join(LOG_FOLDER);
    let app_name = env!("CARGO_PKG_NAME");
    init_simple_logger(app_name, log_folder);
}
