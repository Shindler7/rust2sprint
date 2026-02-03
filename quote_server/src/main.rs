//! Quote Server. Консольное приложение генерации котировок о ценах акций.

#![warn(missing_docs)]

mod channels;
mod cli;
mod config;
mod generator;
mod models;
mod tcp;
mod udp;

use cli::parse_cli_args;
use commons::{errors::QuoteError, init_simple_logger, utils::get_workspace_root};
use config::LOG_FOLDER;
use log::{error, info, warn};
use std::{io, process::exit};
use tcp::run_server;

fn main() -> io::Result<()> {
    if let Err(err) = init_logger() {
        error!("{}", err);
        exit(1);
    }

    info!("Инициализация Quote Server...");

    let cli_args = parse_cli_args();
    info!("Конфигурация получена: {:?}", cli_args);

    if let Err(err) = run_server(cli_args) {
        error!("Сервер остановился с ошибкой: {err}");
    }

    info!("Сервер остановлен.");
    Ok(())
}

/// Инициализировать логгер приложения.
///
/// Используется метод [`init_simple_logger`] из коробки [`commons`].
fn init_logger() -> Result<(), QuoteError> {
    let log_folder = get_workspace_root().join(LOG_FOLDER);
    let app_name = env!("CARGO_PKG_NAME");
    init_simple_logger(app_name, log_folder)?;

    Ok(())
}
