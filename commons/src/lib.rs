use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub mod errors;
pub mod models;
pub mod randomizer;
pub mod utils;

/// Предоставить вектор с наименованием тикеров из файла.
///
/// ## Пример
///
/// ```
/// use commons::utils::get_workspace_root;
/// use commons::get_ticker_data;
///
/// let path_to_file = get_workspace_root().join("data").join("tickers.txt");
/// let data = get_ticker_data(&path_to_file);
///
/// println!("Data: {:?}", data);
/// ```
///
/// ## Returns
///
/// Возвращает вектор при успешной подгрузке данных или None, если вектор
/// получился пустой.
///
/// Паникует при невозможности извлечь данные.
pub fn get_ticker_data(path: &PathBuf) -> Option<Vec<String>> {
    let file = File::open(path).unwrap_or_else(|e| panic!("Не удалось открыть {:?}: {e}", path));

    let tickers: Vec<String> = BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if tickers.is_empty() {
        return None;
    }
    Some(tickers)
}
