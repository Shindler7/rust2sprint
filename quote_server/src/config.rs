//! Конфигурация приложения.

use commons::get_ticker_data;
use commons::utils::get_workspace_root;
use std::sync::LazyLock;

/// Название каталога для хранения данных проекта.
const DATA_FOLDER: &str = "data";
/// Название директории для log-файлов.
pub const LOG_FOLDER: &str = "log";

/// Название файла, который содержит названия тикеров.
const TICKERS_FILENAME: &str = "tickers.txt";

/// Исходный вектор с наименованием тикеров.
pub static TICKER_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let path = get_workspace_root()
        .join(DATA_FOLDER)
        .join(TICKERS_FILENAME);

    if let Some(tickers) = get_ticker_data(&path) {
        tickers
    } else {
        panic!("Отсутствуют данные о тикерах!")
    }
});

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

Важно: отправка новой команды отменяет прежнюю.

"#;

/// Адрес сервера для подключения клиентов.
pub const SERVER_ADDRESS: &str = "127.0.0.1";

/// Порт TCP, на котором сервер принимает подключения.
pub const SERVER_PORT: u16 = 8888;

/// Адрес сервера в формате `адрес:порт`.
pub fn server_endpoint() -> String {
    format!("{}:{}", SERVER_ADDRESS, SERVER_PORT)
}
