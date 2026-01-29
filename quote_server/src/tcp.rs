//! Механизация TCP-сервера.

use crate::config::{WELCOME_INFO, WELCOME_SERVER};
use commons::errors::QuoteError;
use macros::QuoteEnumDisplay;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;

trait WriteExt {
    /// Отправляет в `Write` переданную текстовую строку, преобразуя её
    /// в байтовую.
    fn write_str(&mut self, s: impl AsRef<str>);
    /// Обёртка для `writer.flush()`, скрывающая обработку `Result`.
    fn flush_ext(&mut self);
}

impl<W: Write> WriteExt for W {
    fn write_str(&mut self, s: impl AsRef<str>) {
        let _ = self.write_all(s.as_ref().as_bytes());
        self.flush_ext()
    }

    fn flush_ext(&mut self) {
        let _ = self.flush();
    }
}

/// Перечисление возможных команд для сервера.
#[derive(QuoteEnumDisplay)]
enum Commands {
    /// Запрос на получение информации о тикерах.
    #[str("stream")]
    Stream,
}

/// Обработчик входных данных соединения.
pub fn handle_client(stream: TcpStream) {
    let mut writer = stream.try_clone().expect("Ошибка клонирования TcpStream");
    let mut reader = BufReader::new(stream);

    writer.write_str(WELCOME_INFO);
    writer.write_str(WELCOME_SERVER);

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => return,

            Ok(_) => {
                execute_command(&mut line, &mut writer);
            }

            Err(_) => {
                todo!()
            }
        }
    }
}

/// Обработка команд, полученных сервером.
fn execute_command(command: &mut str, writer: &mut TcpStream) {
    let input = command.trim();
    if input.is_empty() {
        writer.flush_ext();
        return;
    }

    todo!()
}
