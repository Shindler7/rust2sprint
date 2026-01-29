//! Универсальные утилиты.

use std::path::PathBuf;
use std::time::SystemTime;

/// Возвращает количество секунд от начала эпохи UNIX, на основе системного
/// времени.
///
/// Время формата POSIX/UNIX: не включает високосные секунды, а каждый день
/// имеет равную длину в 86400 секунд.
///
/// Возможна паника, если системные часы выставлены на время ранее
/// 1 января 1970 года 0:00:00 UTC.
pub fn get_timestamp() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => panic!("Системное время раньше 01.01.1970 0:00:00 UTC"),
    }
}

/// Предоставить родительский каталог проекта.
///
/// Для `debug` это будет директория расположения `cargo.toml`, а для `release`
/// расположение скомпилированного файла.
#[cfg(debug_assertions)]
pub fn get_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[cfg(not(debug_assertions))]
pub fn get_project_root() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .expect("Не удалось определить путь к исполняемому файлу")
}

/// Предоставить корневую директорию всего проекта.
///
/// В зависимости от статуса проекта предоставляет путь к корневой директории
/// `workspace`, а для `release` к месту расположения скомпилированного файла,
/// что также является корневым путём.
///
/// Вызывает панику при неудачах определения путей.
pub fn get_workspace_root() -> PathBuf {
    let project_root = get_project_root();
    if cfg!(debug_assertions) {
        project_root
            .parent()
            .expect("Не удалось получить родительский каталог workspace")
            .to_path_buf()
    } else {
        project_root.to_path_buf()
    }
}
