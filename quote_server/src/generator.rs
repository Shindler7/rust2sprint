//! Инструменты для генерации данных.

use crate::config::*;
use commons::errors::QuoteError;
use commons::get_ticker_data;
use commons::models::{StockQuote, Transaction};
use commons::randomizer::{random_bool, random_by_tuple, random_choice_str, shuffle_vec};
use commons::utils::{get_timestamp, get_workspace_root};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Генератор котировок тикеров.
///
/// ## Доступные методы
///
/// - [`QuoteGenerator::new`] — создание и настройка экземпляра
/// - [`QuoteGenerator::next_gen`] — генерация нового тикера, с обновлением
///   "табло котировок". Параметры генерации задаются в настройках приложения.
///
/// ## Пример
///
/// ```
/// use crate::generator::QuoteGenerator;
///
/// let generator = QuoteGenerator::new().unwrap();
///
/// let quote1 = generator.next_gen().unwrap();
/// let quote2 = generator.next_gen().unwrap();
///
/// println!("{}", quote1);
/// println!("{}", quote2);
/// ```
pub struct QuoteGenerator {
    /// Данные об известных тикерах (например, загруженные из файла).
    ticker_data: HashSet<String>,
    /// Актуальное состояние "доски котировок" тикеров.
    quote_board: Arc<Mutex<HashMap<String, f64>>>,
}

impl QuoteGenerator {
    /// Инициализация генератора. Проверка данных и их адаптация для работы
    /// генератора.
    pub fn new() -> Result<Self, QuoteError> {
        let tickers_vec = Self::get_ticker_data()?;
        let quote_board = Self::init_quote_board(tickers_vec.clone());
        let ticker_data = tickers_vec.into_iter().collect::<HashSet<String>>();

        let generator = Self {
            ticker_data,
            quote_board,
        };

        Ok(generator)
    }

    /// Загрузить данные по тикерам из файла, в соответствии с параметрами,
    /// указанными в конфигурации.
    ///
    /// ## Используются:
    ///
    /// - [`DATA_FOLDER`] — директория для хранения файлов с данными
    /// - [`TICKERS_FILENAME`] — название файла с данными о тикерах.
    ///
    /// ## Returns
    ///
    /// Вектор с названиями тикеров.
    pub fn get_ticker_data() -> Result<Vec<String>, QuoteError> {
        let tickers_file = get_workspace_root()
            .join(DATA_FOLDER)
            .join(TICKERS_FILENAME);

        get_ticker_data(&tickers_file)?
            .ok_or_else(|| QuoteError::ticker_err("отсутствуют данные по тикерам"))
    }

    /// Инициализация "табло котировок".
    ///
    /// Формирует первичные значения на основе настроек приложения.
    fn init_quote_board(tickers: Vec<String>) -> Arc<Mutex<HashMap<String, f64>>> {
        let settings = QUOTE_SETTINGS;

        let shuffle_tickers = shuffle_vec(tickers);

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

        Arc::new(Mutex::new(map))
    }

    /// Сформировать экземпляр на основе предустановленных в конфигурации
    /// значений ([`QUOTE_SETTINGS`]).
    ///
    /// При генерации новой цены она сохраняется для выбранного тикера
    /// в "табло котировок".
    pub fn next_gen(&mut self) -> Result<StockQuote, QuoteError> {
        let ticker = random_choice_str(&self.ticker_data)
            .ok_or_else(|| QuoteError::ticker_err("неудачная попытка случайного выбора тикера"))?;
        let price = self.update_price_random(&ticker)?;
        let volume: u32 = random_by_tuple(QUOTE_SETTINGS.units_per_trade);
        let transaction = if random_bool(0.5) {
            Transaction::Sell
        } else {
            Transaction::Buy
        };

        let new_quote = Self::new_quote(ticker, price, volume, transaction);

        Ok(new_quote)
    }

    /// Создать новый экземпляр [`StockQuote`] с предоставленными значениями.
    fn new_quote(ticker: String, price: f64, volume: u32, transaction: Transaction) -> StockQuote {
        let timestamp = get_timestamp();

        StockQuote {
            ticker,
            price,
            volume,
            transaction,
            timestamp,
        }
    }

    /// Обновить стоимость тикера в табло котировок.
    ///
    /// Для равномерности изменения цен, предусмотрен механизм плавного случайного
    /// колебания цены (+/- 10 % от предыдущей), но в пределах установленных
    /// настройками.
    ///
    /// ## Ошибки
    ///
    /// Возвращает [`QuoteError::LockError`] если возникла ошибка блокировки
    /// доступа к данным.
    fn update_price_random(&mut self, ticker: &str) -> Result<f64, QuoteError> {
        let old_price = self.read_price(ticker)?;

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
        self.write_price(ticker, new_price)?;
        Ok(new_price)
    }

    /// Считать стоимость по заданному тикеру.
    fn read_price(&self, ticker: &str) -> Result<f64, QuoteError> {
        self.quote_board
            .lock()?
            .get(ticker)
            .copied()
            .ok_or_else(|| QuoteError::ticker_err(format!("тикер {} не найден", ticker)))
    }

    /// Сохранить новое значение стоимости для тикера.
    fn write_price(&mut self, ticker: &str, price: f64) -> Result<(), QuoteError> {
        self.quote_board.lock()?.insert(ticker.to_string(), price);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_initializes() {
        let generator = QuoteGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn generator_produces_quote() {
        let mut generator = QuoteGenerator::new().unwrap();
        let quote = generator.next_gen().unwrap();

        assert!(!quote.ticker.is_empty());
        assert!(quote.price > 0.0);
        assert!(quote.volume > 0);
    }

    #[test]
    fn generated_ticker_is_known() {
        let mut generator = QuoteGenerator::new().unwrap();
        let tickers = QuoteGenerator::get_ticker_data().unwrap();

        let quote = generator.next_gen().unwrap();
        assert!(tickers.contains(&quote.ticker));
    }
}
