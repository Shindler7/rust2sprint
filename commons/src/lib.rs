use crate::errors::QuoteError;
use log::*;
use simplelog::{CombinedLogger, Config, WriteLogger};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub mod errors;
pub mod models;
pub mod randomizer;
pub mod traits;
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
/// let data = get_ticker_data(&path_to_file).unwrap();
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
pub fn get_ticker_data(path: &PathBuf) -> Result<Option<Vec<String>>, QuoteError> {
    let file = File::open(path).map_err(|err| {
        QuoteError::ticker_err(format!(
            "Не удалось открыть файл с тикерами {}: {}",
            path.to_string_lossy(),
            err
        ))
    })?;

    let tickers: Vec<String> = BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if tickers.is_empty() {
        return Ok(None);
    }
    Ok(Some(tickers))
}

/// Фабрика по созданию индивидуальных логгеров для приложений.
///
/// Инициализация требуется один раз при запуске приложения. Далее используются
/// стандартные макросы коробки `log`: `info`, `warn`, `error` для
/// логирования событий.
///
/// ## Args
///
/// - `app_name` — название приложения (будет использовано для создания файла)
/// - `log_dir` — путь к директории расположения log-файлов (при отсутствии
///   пытается создать)
///
/// ## Пример
///
/// ```no_run
/// use log::*;
/// use commons::init_simple_logger;
/// use commons::utils::get_workspace_root;
/// use std::path::PathBuf;
///
/// let log_dir = get_workspace_root().join("log");
/// init_simple_logger("app_name", log_dir);
///
/// info!("Всё в порядке");
/// warn!("Предупреждаем: погода портиться!");
/// error!("Шторм разрушил усадьбу, сэр!");
/// ```
///
/// Возвращает ошибки создания (открытия) директории и (или) log-файла,
/// и при инициализации логгера (предоставляет сообщение о причинах, если
/// есть).
pub fn init_simple_logger(app_name: &str, log_dir: PathBuf) -> Result<(), QuoteError> {
    let config = Config::default();
    let log_file_path = log_dir.join(format!("{}.log", app_name));

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).map_err(|_| {
            QuoteError::runtime_err(format!(
                "не удалось сформировать путь: {}",
                log_dir.display()
            ))
        })?;
    }

    let log_file = File::create(&log_file_path).map_err(|_| {
        QuoteError::runtime_err(format!(
            "ошибка работы с log-файлом: {}",
            log_file_path.display()
        ))
    })?;

    let logger = WriteLogger::new(LevelFilter::Info, config, log_file);

    CombinedLogger::init(vec![logger])
        .map_err(|e| QuoteError::runtime_err(format!("ошибка инициализации логгера: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_ticker_data_with_valid_file() {
        // Создаем временный файл с корректными данными
        let mut temp_file = NamedTempFile::new().expect("Не удалось создать временный файл");
        writeln!(temp_file, "AAPL\nGOOGL\nMSFT\nTSLA\nAMZN").expect("Не удалось записать в файл");

        let path = temp_file.path().to_path_buf();
        let result = get_ticker_data(&path);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let tickers = result.unwrap();
        assert_eq!(tickers.len(), 5);
        assert_eq!(tickers, vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN"]);
    }

    #[test]
    fn test_get_ticker_data_with_empty_and_whitespace() {
        // Создаем временный файл с пустыми строками, пробелами и корректными данными
        let mut temp_file = NamedTempFile::new().expect("Не удалось создать временный файл");
        writeln!(temp_file, "AAPL\n\n  GOOGL  \n\nMSFT\n\t\nAMZN")
            .expect("Не удалось записать в файл");

        let path = temp_file.path().to_path_buf();
        let result = get_ticker_data(&path);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let tickers = result.unwrap();
        assert_eq!(tickers.len(), 4); // Пустые строки должны быть отфильтрованы
        assert_eq!(tickers, vec!["AAPL", "GOOGL", "MSFT", "AMZN"]);
    }

    #[test]
    fn test_get_ticker_data_with_nonexistent_file() {
        // Передан некорректный путь к файлу.
        let non_existent_path = PathBuf::from("/non/existent/path/tickers.txt");
        let result = get_ticker_data(&non_existent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_ticker_data_with_empty_file() {
        // Создаем пустой временный файл
        let temp_file = NamedTempFile::new().expect("Не удалось создать временный файл");
        let path = temp_file.path().to_path_buf();
        let result = get_ticker_data(&path);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_none());
    }
}
