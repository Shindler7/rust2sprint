//! Quote Server. Консольное приложение генерации котировок о ценах акций.

#![warn(missing_docs)]

mod cli;
mod config;
mod generator;
mod tcp;
mod udp;

use cli::parse_cli_args;
use commons::{init_simple_logger, utils::get_workspace_root};
use config::{GEN_TICKERS_DURATION_MS, LOG_FOLDER};
use crossbeam_channel::{unbounded, SendTimeoutError, Sender};
use generator::QuoteGenerator;
use log::{error, info, warn};
use std::{io, thread, time::Duration};
use tcp::run_server;

fn main() -> io::Result<()> {
    init_logger();

    info!("Инициализация Quote Server...");

    let cli_args = parse_cli_args();
    info!("Конфигурация получена: {:?}", cli_args);

    let (quote_tx, quote_rx) = unbounded();
    start_generator(quote_tx);

    if let Err(err) = run_server(cli_args, quote_rx) {
        error!("Сервер остановился с ошибкой: {err}");
    }

    info!("Сервер остановлен.");
    Ok(())
}

/// Инициализировать логгер приложения.
///
/// Используется метод [`init_simple_logger`] из коробки [`commons`].
fn init_logger() {
    let log_folder = get_workspace_root().join(LOG_FOLDER);
    let app_name = env!("CARGO_PKG_NAME");
    init_simple_logger(app_name, log_folder);
}

/// Запустить ленту котировок.
fn start_generator(tx: Sender<String>) {
    thread::spawn(move || {
        let mut generator =
            QuoteGenerator::new().unwrap_or_else(|err| panic!("ошибка генератора: {err}"));

        info!("Генератор котировок запущен");

        loop {
            thread::sleep(Duration::from_millis(GEN_TICKERS_DURATION_MS));

            if let Ok(quote) = generator.next_gen() {
                match tx.send_timeout(
                    quote.to_string(),
                    Duration::from_millis(GEN_TICKERS_DURATION_MS),
                ) {
                    Ok(_) => (),
                    Err(SendTimeoutError::Timeout(_)) => {
                        warn!("Канал котировок занят (timeout)");
                    }
                    Err(SendTimeoutError::Disconnected(_)) => {
                        error!("Канал котировок закрыт");
                        break;
                    }
                }
            }
        }

        info!("Генератор котировок остановлен");
    });
}
