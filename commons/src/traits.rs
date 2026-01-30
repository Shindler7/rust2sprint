//! Универсальные трейты для приложений Quote Server и Quote Client.

use std::io::Write;

pub trait WriteExt {
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
