//! Инструменты для генерации данных.

use crate::config::{QuoteGenerateSettings, QUOTE_SETTINGS, TICKER_DATA};
use commons::errors::QuoteError;
use commons::models::{StockQuote, Transaction};
use commons::randomizer::{random_bool, random_by_tuple, random_choice_str, shuffle_vec};
use commons::utils::get_timestamp;
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

fn init_quote_board(settings: QuoteGenerateSettings, tickers: &[String]) -> HashMap<String, f64> {
    let shuffle_tickers = shuffle_vec(tickers.to_vec());

    let total = shuffle_tickers.len();
    let expensive_count = ((total as f64) * settings.top_share).ceil() as usize;
    let middle_count = ((total as f64) * settings.middle_share).ceil() as usize;

    let mut map = HashMap::with_capacity(total);

    for (i, ticker) in shuffle_tickers.into_iter().enumerate() {
        let price = if i < expensive_count {
            random_by_tuple(settings.expensive)
        } else if i < expensive_count + middle_count {
            random_by_tuple(settings.middle)
        } else {
            random_by_tuple(settings.low)
        };

        map.insert(ticker, price);
    }

    map
}

/// Обновить стоимость тикера в [`QUOTE_BOARD`].
///
/// Для равномерности изменения цен, предусмотрен механизм плавного случайного
/// колебания цены (+/- 10 % от предыдущей), но в пределах установленных
/// настройками.
fn update_price_random(ticker: &str) -> Result<f64, QuoteError> {
    let old_price = read_price(ticker)?;

    // Цена меняется?
    if !random_bool(QUOTE_SETTINGS.probability_change_price) {
        return Ok(old_price);
    }

    let calc_min = old_price * 90.0 / 100.0;
    let calc_max = old_price * 110.0 / 100.0;

    let range = match (
        calc_min < QUOTE_SETTINGS.low.0,
        calc_max > QUOTE_SETTINGS.expensive.1,
    ) {
        (true, false) => (QUOTE_SETTINGS.low.0, QUOTE_SETTINGS.low.1),
        (false, true) => (QUOTE_SETTINGS.expensive.0, QUOTE_SETTINGS.expensive.1),
        _ => (calc_min, calc_max),
    };

    let new_price = random_by_tuple(range);
    write_price(ticker, new_price)?;
    Ok(new_price)
}

/// Виртуальное табло котировок тикеров.
///
/// При запуске приложения список тикеров перемешивается и делится на 3 группы:
/// - дорогие (составляют 10 %)
/// - средние (наиболее ходовые) от 10 до 50 %
/// - низовые (оставшаяся половина).
///
/// С учётом настроек приложения выставляются первичные значения стоимости
/// каждого тикера, и впоследствии они обновляются при генерации событий.
static QUOTE_BOARD: LazyLock<Mutex<HashMap<String, f64>>> =
    LazyLock::new(|| Mutex::new(init_quote_board(QUOTE_SETTINGS, &TICKER_DATA)));

/// Считать стоимость по заданному тикеру.
fn read_price(ticker: &str) -> Result<f64, QuoteError> {
    QUOTE_BOARD
        .lock()?
        .get(ticker)
        .copied()
        .ok_or_else(|| QuoteError::ticker_err(format!("тикер {} не найден", ticker)))
}

/// Сохранить новое значение стоимости для тикера.
fn write_price(ticker: &str, price: f64) -> Result<(), QuoteError> {
    QUOTE_BOARD.lock()?.insert(ticker.to_string(), price);
    Ok(())
}

pub struct QuoteGenerator {
    pub quote: StockQuote,
}

impl QuoteGenerator {
    /// Создать новый экземпляр [`StockQuote`] с предоставленными значениями.
    pub fn new(ticker: String, price: f64, volume: u32, transaction: Transaction) -> Self {
        let timestamp = get_timestamp();
        let quote = StockQuote {
            ticker,
            price,
            volume,
            transaction,
            timestamp,
        };

        Self { quote }
    }

    /// Сформировать экземпляр на основе предустановленных в конфигурации
    /// значений (см. [`QUOTE_SETTINGS`]) и табло котировок ([`QUOTE_BOARD`]).
    ///
    /// Для равномерности изменения цен, предусмотрен механизм плавного
    /// увеличения или уменьшения цены (+/- 10 %), но в пределах установленных
    /// настройками.
    ///
    /// При генерации новой цены она сохраняется для выбранного тикера
    /// в [`QUOTE_BOARD`].
    pub fn generate() -> Result<Self, QuoteError> {
        let ticker = match random_choice_str(TICKER_DATA.as_slice()) {
            Some(ticker) => ticker,
            None => panic!("Неверное поведение: отсутствуют данные по тикетам"),
        };
        let price = update_price_random(&ticker)?;
        let volume: u32 = random_by_tuple(QUOTE_SETTINGS.units_per_trade);
        let transaction = if random_bool(0.5) {
            Transaction::Sell
        } else {
            Transaction::Buy
        };

        Ok(Self::new(ticker, price, volume, transaction))
    }
}
