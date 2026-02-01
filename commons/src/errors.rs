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
    #[error("{0}")]
    ValueError(String),

    /// Ошибка генерации тикеров.
    #[error("ошибка при формировании тикера: {0}")]
    TickerError(String),

    /// Ошибка блокировки mutex.
    #[error("ошибка блокировки: {0}")]
    LockError(String),

    // Ошибки при работе сервера.
    #[error("{0}")]
    ServerError(String),

    /// Некорректная команда серверу.
    #[error("{0}")]
    CommandError(String),
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

    /// Конструктор для ошибки [`QuoteError::CommandError`].
    pub fn command_err(message: impl Into<String>) -> QuoteError {
        Self::CommandError(message.into())
    }

    /// Конструктор для ошибки [`QuoteError::ServerError`].
    pub fn server_err(message: impl Into<String>) -> QuoteError {
        Self::ServerError(message.into())
    }
}
