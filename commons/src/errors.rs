//! Собственные типы ошибок приложения.
//!
//! Для поддержки функциональности применяется крейт `thiserror`.

use thiserror::Error;

/// Дерево ошибок приложений Quote.
#[derive(Error, Debug)]
pub enum QuoteError {
    /// Некорректное значение.
    ///
    /// Например, если ожидается число в диапазоне от 1 до 10, а передано 15.
    #[error("неверное значение: {0}")]
    ValueError(String),
}

impl QuoteError {
    pub fn value_err(message: impl Into<String>) -> QuoteError {
        Self::ValueError(message.into())
    }
}
