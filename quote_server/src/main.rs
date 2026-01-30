//! Quote Server. Консольное приложение генерации котировок о ценах акций.
//! Например, для тикеров "AAPL", "GOOGL", "TSLA". Данные включают ряд
//! параметров, которые можно дополнять.

#![warn(missing_docs)]

use commons::utils::get_workspace_root;
use log::{error, info};
use std::net::TcpListener;
use std::thread;

mod config;
mod generator;
mod tcp;
mod udp;

use crate::tcp::handle_client;
use commons::init_simple_logger;
use config::{server_endpoint, LOG_FOLDER};

fn main() -> std::io::Result<()> {
    // Инициализация логгера.
    init_logger();

    let endpoint = server_endpoint();
    let listener = TcpListener::bind(&endpoint)?;
    println!("Запущен сервер по адресу {}", &endpoint);
    println!("Завершить работу сервера с помощью CTRL-C/CTRL-BREAK.\n");

    info!("Quote Server запущен");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("Рукопожатие: {:?}", stream);
                thread::spawn(|| handle_client(stream));
            }
            Err(e) => {
                error!("Ошибка работы сервера: {}", e);
            }
        }
    }

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
