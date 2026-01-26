//! Инструменты для генерации данных.

use crate::config::{PRICE_MIN_MAX, TICKER_DATA, VOLUME_MIN_MAX};
use commons::errors::QuoteError;
use commons::randomizer::{random, random_choice_str};
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

    /// Создать экземпляр со случайными значениями.
    ///
    /// Временная метка соответствует системному времени.
    pub fn generate_new() -> Self {
        let ticker = match random_choice_str(TICKER_DATA.as_slice()) {
            Some(ticker) => ticker,
            None => panic!("Неверное поведение: отсутствуют данные по тикетам"),
        };
        let price: f64 = random(PRICE_MIN_MAX.0, PRICE_MIN_MAX.1);
        let volume: u32 = random(VOLUME_MIN_MAX.0, VOLUME_MIN_MAX.1);

        Self::new(ticker, price, volume)
    }
}
