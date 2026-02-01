//! Quote Client. Приложение для взаимодействия с Quote Server.

use log::{info, warn};
use std::{
    io::{BufRead, BufReader, Result, Write},
    net::TcpStream,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
};

mod cli;
mod config;
mod udp;

use cli::parse_cli_args;
use commons::{init_simple_logger, utils::get_workspace_root};
use config::LOG_FOLDER;

fn main() -> Result<()> {
    init_logger();
    let client_set = parse_cli_args();

    info!("Quote Client запущен");

    let stream = TcpStream::connect(client_set.server_addr)
        .unwrap_or_else(|e| panic!("Ошибка подключения к {}: {}", client_set.server_addr, e));

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    info!(
        "Установлено соединение с сервером: {}",
        client_set.server_addr
    );

    // Пропуск приветствия и служебной информации.
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if line.trim_end().to_uppercase() == "READY" {
            break;
        }
    }

    writer.write_all(client_set.command.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;

    info!("Отправлена команда: {}", client_set.command);

    let mut server_response = String::new();
    let bytes = reader.read_line(&mut server_response)?;
    if bytes == 0 {
        let err_msg = "Пустой ответ от сервера или сервер закрыл соединение.";
        warn!("{}", err_msg);
        return Ok(());
    }

    let response = server_response.trim_end();
    info!("Ответ сервера: {}", response);

    if !response.starts_with("OK") {
        warn!("Сервер отклонил команду");
        return Ok(());
    }

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    ctrlc::set_handler(move || {
        stop_flag_clone.store(true, Ordering::SeqCst);
    })
    .expect("Ошибка установки Ctrl-C");

    let udp = udp::UdpClient::bind_url(&client_set.udp_url)?;
    let ping_handle = udp.spawn_ping(stop_flag.clone());

    udp.recv_loop(stop_flag);
    let _ = ping_handle.join();

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
