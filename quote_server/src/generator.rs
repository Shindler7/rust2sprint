//! Инструменты для генерации данных.

use commons::errors::QuoteError;
use commons::utils::get_timestamp;
use macros::QuoteDisplay;

/// Структура биржевой котировки.
#[derive(Debug, Clone, QuoteDisplay)]
pub struct StockQuote {
    /// Короткое наименование биржевого инструмента (тикер).
    pub ticker: String,
    /// Текущая цена за единицу.
    pub price: f64,
    /// Количество приобретённых акций.
    pub volume: u32,
    /// Временная метка операции.
    pub timestamp: u64,
}

impl StockQuote {
    /// Создать новый экземпляр.
    ///
    /// Временная метка проставляется автоматически.
    pub fn new(ticker: String, price: f64, volume: u32) -> Self {
        let timestamp = get_timestamp();
        Self {
            ticker,
            price,
            volume,
            timestamp,
        }
    }

    pub fn generate() -> Self {
        todo!()
    }
}
