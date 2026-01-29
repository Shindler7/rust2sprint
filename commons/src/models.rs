//! Модели данных для приложений.

use crate::errors::QuoteError;
use macros::{QuoteDisplay, QuoteEnumDisplay};

/// Вид транзакций для биржевого события.
#[derive(Debug, Clone, QuoteEnumDisplay)]
pub enum Transaction {
    /// Продажа.
    #[str("sell")]
    Sell,
    /// Покупка.
    #[str("buy")]
    Buy,
}

/// Структура биржевого события.
#[derive(Debug, Clone, QuoteDisplay)]
pub struct StockQuote {
    /// Короткое наименование биржевого инструмента (тикер).
    pub ticker: String,
    /// Текущая цена за единицу.
    pub price: f64,
    /// Количество приобретённых (проданных) акций.
    pub volume: u32,
    /// Временная метка операции.
    pub timestamp: u64,
    /// Вид транзакции.
    pub transaction: Transaction,
}
