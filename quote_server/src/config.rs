//! Конфигурация приложения.

use std::ops::RangeInclusive;

/// Название каталога для хранения данных проекта.
pub const DATA_FOLDER: &str = "data";
/// Название директории для log-файлов.
pub const LOG_FOLDER: &str = "log";

/// Название файла, который содержит названия тикеров.
pub const TICKERS_FILENAME: &str = "tickers.txt";

/// Настройки генератора стоимости тикеров.
#[derive(Clone, Copy)]
pub struct QuoteGenerateSettings {
    /// Диапазон цены дорогих тикеров (верхние, например, 10 %).
    pub expensive: (f64, f64),
    /// Диапазон цены тикеров средней стоимости (от 10 до 50 %).
    pub middle: (f64, f64),
    /// Стоимость тикеров низшего эшелона (50 % и более).
    pub low: (f64, f64),

    /// Доля "топ" тикеров (по умолчанию 0.1 = 10 %).
    pub top_share: f64,
    /// Доля "средних" тикеров (по умолчанию 0.4 = 40 %).
    pub middle_share: f64,

    /// Диапазон возможных значений объёма разовой сделки с тикерами.
    pub units_per_trade: (u32, u32),
    /// Вероятность изменения цены при очередной генерации. Возможное значение
    /// от 0 до 1 (где 0 всегда `false`, а 1 всегда `true`).
    pub probability_change_price: f64,
}

/// Предустановленные значения [`QuoteGenerateSettings`].
pub static QUOTE_SETTINGS: QuoteGenerateSettings = QuoteGenerateSettings {
    expensive: (500.0, 1500.0),
    middle: (100.0, 499.0),
    low: (0.5, 99.9),
    top_share: 0.10,
    middle_share: 0.40,
    units_per_trade: (1, 500_000),
    probability_change_price: 0.9,
};

pub const WELCOME_SERVER: &str = "Успешное подключение к Quote Server!\n\n";
pub const WELCOME_INFO: &str = r#"Commands:
1. Получать данные о всех тикерах:
STREAM <URL>:<PORT> ALL
 Пример: udp://127.0.0.1:34254 ALL

2. Получать данные по отдельным тикерам:
STREAM <URL>:<PORT> <TICKERS, ...>
 Пример: udp://127.0.0.1:34254 PSA,EMR,DUK,PYPL
 Ошибки: неверные имена тикеров

3. Отменить ранее заказанную отправку данных:
CANCEL <URL>:<PORT>

Важно: отправка новой команды БЕЗ ОТМЕНЫ (CANCEL) вернёт ошибку.

"#;

/// Строка-терминатор после приветствия сервера.
pub const WELCOME_TERMINATOR: &str = "READY\n";

/// Адрес сервера для подключения клиентов.
pub const SERVER_ADDRESS: [u8; 4] = [127, 0, 0, 1];

/// Порт TCP, на котором сервер принимает подключения.
pub const DEFAULT_SERVER_PORT: u16 = 8888;

/// Допустимые значения порта TCP.
pub const TCP_PORTS_ALLOWED: RangeInclusive<usize> = 1024..=49151;

/// Интервал между генерациями тикеров.
pub const GEN_TICKERS_DURATION_MS: u64 = 100;

/// Лимит времени ожидания пинга от клиента (в секундах).
pub const UDP_PING_TIMEOUT_SECS: u64 = 5;

/// Timeout ожидания сообщения из канала тикеров (миллисекунды).
pub const CHANNEL_TIMEOUT_MS: u64 = 200;

/// Timeout на операцию чтения из UDP-сокета (миллисекунды).
pub const SOCKET_READ_TIMEOUT_MS: u64 = 500;
