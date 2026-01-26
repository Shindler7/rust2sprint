//! Конфигурация приложения.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::LazyLock;

/// Название каталога для хранения данных проекта.
pub const DATA_FOLDER: &str = "data";

/// Название файла, который содержит названия тикеров.
pub const TICKERS_FILENAME: &str = "tickers.txt";

pub static TICKER_DATA: LazyLock<Vec<String>> = LazyLock::new(|| {
    let path = get_project_root().join(DATA_FOLDER).join(TICKERS_FILENAME);
    let file = File::open(&path).unwrap_or_else(|e| panic!("Не удалось открыть {:?}: {e}", path));

    BufReader::new(file)
        .lines()
        .filter_map(Result::ok)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
});

/// Диапазон возможных значений стоимости тикетов.
pub const PRICE_MIN_MAX: (f64, f64) = (1000.0, 1_000_000.0);

/// Диапазон возможных значений объёма ценных бумаг.
pub const VOLUME_MIN_MAX: (u32, u32) = (1, 500_000);

/// Предоставить родительский каталог проекта.
pub fn get_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
