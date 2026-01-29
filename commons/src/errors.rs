//! Собственные типы ошибок приложения.
//!
//! Для поддержки функциональности применяется крейт `thiserror`.

use std::sync::PoisonError;
use thiserror::Error;

/// Дерево ошибок приложений Quote.
#[derive(Error, Debug)]
pub enum QuoteError {
    /// Некорректное значение.
    ///
    /// Например, если ожидается число в диапазоне от 1 до 10, а передано 15.
    #[error("неверное значение: {0}")]
    ValueError(String),

    /// Ошибка генерации тикеров.
    #[error("ошибка при формировании тикера: {0}")]
    TickerError(String),

    /// Ошибка блокировки mutex.
    #[error("ошибка блокировки: {0}")]
    LockError(String),
}

impl<T> From<PoisonError<T>> for QuoteError {
    fn from(err: PoisonError<T>) -> Self {
        QuoteError::LockError(err.to_string())
    }
}

impl QuoteError {
    /// Конструктор для ошибки [`QuoteError::ValueError`].
    pub fn value_err(message: impl Into<String>) -> QuoteError {
        Self::ValueError(message.into())
    }

    /// Конструктор для ошибки [`QuoteError::TickerError`].
    pub fn ticker_err(message: impl Into<String>) -> QuoteError {
        Self::TickerError(message.into())
    }
}
