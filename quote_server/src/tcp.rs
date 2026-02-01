//! Механизация TCP-сервера.

use crate::cli::ServerSet;
use crate::config::{WELCOME_INFO, WELCOME_SERVER, WELCOME_TERMINATOR};
use crate::generator::QuoteGenerator;
use crate::udp::spawn_stream;
use commons::{errors::QuoteError, traits::WriteExt};
use crossbeam_channel::Receiver;
use log::{error, info};
use macros::QuoteEnumDisplay;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering}, Arc,
    Mutex,
};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    io,
    io::{BufRead, BufReader},
    net::{SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    thread::sleep,
    time::Duration,
};
use url::Url;

/// Счётчик клиентов.
static CLIENTS_COUNTER: AtomicUsize = AtomicUsize::new(1000);

/// Увеличить значение счётчика клиентов и вернуть уникальное значение.
fn gen_id() -> usize {
    CLIENTS_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Тип ответа сервера клиенту.
enum ServerResponse {
    /// Успешное исполнение команды.
    Ok { message: Option<String> },
    /// Ошибка при выполнении команды.
    Err { message: Option<String> },
}

impl Display for ServerResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerResponse::Ok { message } => match message {
                Some(msg) => write!(f, "OK|{}", msg),
                None => write!(f, "OK"),
            },
            ServerResponse::Err { message } => match message {
                Some(msg) => write!(f, "ERROR|{}", msg),
                None => write!(f, "ERROR"),
            },
        }
    }
}

impl ServerResponse {
    /// Отправить ответ клиенту.
    ///
    /// Пример: `OK|Успешно`.
    ///
    /// ## Args
    ///
    /// - `writer` — TCP-поток для записи ответа
    /// - `addr` — адрес TCP-сокета клиента
    /// - `log` — если `true`, сообщение также записывается в лог-файл
    pub fn send(&self, writer: &mut TcpStream, addr: SocketAddr, log: bool) {
        let response = self.to_string();
        if log {
            info!("Ответ: {} для клиента {}", response, addr);
        }
        writer.write_str(&response);
        writer.flush_ext();
    }

    /// Успешный ответ.
    pub fn ok(message: &str) -> Self {
        if message.trim().is_empty() {
            ServerResponse::Ok { message: None }
        } else {
            ServerResponse::Ok {
                message: Some(message.to_string()),
            }
        }
    }

    /// Ответ с ошибкой.
    pub fn err(message: &str) -> Self {
        if message.trim().is_empty() {
            ServerResponse::Err { message: None }
        } else {
            ServerResponse::Err {
                message: Some(message.to_string()),
            }
        }
    }
}

/// Команды клиента.
#[derive(Debug, QuoteEnumDisplay)]
enum Command {
    /// Подписка на поток.
    #[str("stream")]
    Stream,
    /// Отменить подписку.
    #[str("cancel")]
    Cancel,
}

impl Command {
    /// Создать подписку клиента.
    pub fn make_client(
        &self,
        unique_id: usize,
        tcp_addr: SocketAddr,
        recv: Receiver<String>,
        cmd_parts: Vec<String>,
    ) -> Result<ClientSubscription, QuoteError> {
        match self {
            Command::Stream => {
                if cmd_parts.len() < 2 {
                    return Err(QuoteError::command_err("команда неполная"));
                }

                let udp_url = Url::parse(&cmd_parts[0]).map_err(|err| {
                    QuoteError::command_err(format!(
                        "некорректный udp-адрес '{}': {}",
                        &cmd_parts[0], err
                    ))
                })?;
                if udp_url.scheme() != "udp" {
                    return Err(QuoteError::command_err("поддерживается только UDP"));
                }

                let tickers_set: HashSet<String> = QuoteGenerator::get_ticker_data()
                    .map_err(|_| QuoteError::command_err("отсутствуют тикеры"))?
                    .into_iter()
                    .collect();

                let tickers = match cmd_parts[1].to_uppercase().as_str() {
                    "ALL" => tickers_set,
                    _ => {
                        let input_set: HashSet<String> = cmd_parts[1]
                            .split(',')
                            .map(|s| s.trim().to_uppercase())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if input_set.is_subset(&tickers_set) {
                            input_set
                        } else {
                            return Err(QuoteError::command_err("некорректные тикеры"));
                        }
                    }
                };

                Ok(ClientSubscription::new(
                    unique_id, tcp_addr, udp_url, tickers, recv,
                ))
            }
            _ => Err(QuoteError::value_err(
                "Данный метод не поддерживает этот вариант перечисления",
            )),
        }
    }
}

/// Подписчик на котировки.
#[derive(Debug, Clone)]
pub(crate) struct ClientSubscription {
    /// Уникальный ID сессии.
    pub unique_id: usize,
    /// TCP-адрес клиента.
    pub tcp_addr: SocketAddr,
    /// UDP-адрес для стрима.
    pub udp_url: Url,
    /// Список тикеров.
    pub tickers: HashSet<String>,
    /// Канал котировок.
    pub recv: Receiver<String>,
    /// Флаг остановки.
    pub stop_flag: Arc<AtomicBool>,
}

impl ClientSubscription {
    /// Создать нового клиента.
    pub fn new(
        unique_id: usize,
        tcp_addr: SocketAddr,
        udp_url: Url,
        tickers: HashSet<String>,
        recv: Receiver<String>,
    ) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        Self {
            unique_id,
            tcp_addr,
            udp_url,
            tickers,
            recv,
            stop_flag,
        }
    }
}

/// Менеджер клиентов.
#[derive(Debug, Default)]
struct ClientManager {
    /// `HashMap` активных клиентов, где ключ — уникальный id сессии.
    clients: HashMap<usize, ClientSubscription>,
}

impl ClientManager {
    /// Создать менеджера.
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Проверить, существует ли клиент с предоставленным id.
    fn id_exists(&self, unique_id: usize) -> bool {
        self.clients.contains_key(&unique_id)
    }

    /// Добавить нового клиента.
    fn add_client(&mut self, client: ClientSubscription) -> Result<(), QuoteError> {
        if self.id_exists(client.unique_id) {
            return Err(QuoteError::value_err("Клиент уже существует"));
        }
        self.clients.insert(client.unique_id, client);
        Ok(())
    }

    /// Удалить клиента.
    fn remove_client(&mut self, unique_id: usize) -> Result<ClientSubscription, QuoteError> {
        self.clients
            .remove(&unique_id)
            .ok_or_else(|| QuoteError::command_err("задачи отсутствуют"))
    }
}

/// Организатор работы TCP-сервера.
pub fn run_server(settings: ServerSet, receiver: Receiver<String>) -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .unwrap_or_else(|e| panic!("Ошибка установки Ctrl-C: {}", e));

    let listener = TcpListener::bind(settings.server_addr)?;
    listener.set_nonblocking(true)?;

    println!("Запущен сервер по адресу {}", settings.server_addr);
    println!("Завершить работу сервера с помощью CTRL-C/CTRL-BREAK.\n");
    info!("Quote Server запущен");

    let client_manager = Arc::new(Mutex::new(ClientManager::new()));

    loop {
        if !running.load(Ordering::SeqCst) {
            info!("Работа сервера прервана...");
            break;
        }

        match listener.accept() {
            Ok((stream, addr)) => {
                let recv = receiver.clone();
                let clients = Arc::clone(&client_manager);
                info!("Рукопожатие: {:?}", addr);
                std::thread::spawn(move || handle_client(stream, addr, recv, clients));
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                sleep(Duration::from_millis(50));
            }
            Err(e) => {
                error!("Ошибка работы сервера: {}", e);
                break;
            }
        }
    }

    if let Ok(mut manager) = client_manager.lock() {
        for (_, client) in manager.clients.iter_mut() {
            client.stop_flag.store(true, Ordering::SeqCst);
            info!("Клиент {} деактивирован", client.tcp_addr);
        }
    }

    Ok(())
}

/// Взаимодействие с новым клиентом.
fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    receiver: Receiver<String>,
    clients: Arc<Mutex<ClientManager>>,
) -> io::Result<()> {
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    writer.write_str(WELCOME_SERVER);
    writer.write_str(WELCOME_INFO);
    writer.flush_ext();
    writer.write_str(WELCOME_TERMINATOR);

    let id_client = gen_id();

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => return Ok(()),
            Ok(_) => {
                let input = line.trim();
                if input.is_empty() {
                    ServerResponse::err("empty line").send(&mut writer, addr, false);
                    continue;
                }

                let mut parts: Vec<String> =
                    input.split_whitespace().map(|s| s.to_string()).collect();

                let cmd = parts.remove(0);
                match Command::from_str(&cmd) {
                    Ok(Command::Stream) => {
                        let recv = receiver.clone();
                        let client = match Command::Stream.make_client(id_client, addr, recv, parts)
                        {
                            Ok(c) => c,
                            Err(err) => {
                                ServerResponse::err(err.to_string().as_str()).send(
                                    &mut writer,
                                    addr,
                                    false,
                                );
                                continue;
                            }
                        };

                        if let Ok(mut clients) = clients.lock() {
                            clients.add_client(client.clone()).ok();
                            spawn_stream(client);
                        }

                        ServerResponse::ok("stream started").send(&mut writer, addr, false);
                    }

                    Ok(Command::Cancel) => {
                        if let Ok(mut clients) = clients.lock()
                            && let Ok(client) = clients.remove_client(id_client)
                        {
                            client.stop_flag.store(true, Ordering::SeqCst);
                        }

                        ServerResponse::ok("canceled").send(&mut writer, addr, false);
                    }

                    Err(_) => {
                        ServerResponse::err("invalid command").send(&mut writer, addr, false);
                    }
                }
            }
            Err(_) => {
                error!("Ошибка чтения: '{}' от {}", line.trim_end(), addr);
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn server_response_format_ok() {
        let r1 = ServerResponse::ok("");
        let r2 = ServerResponse::ok("hello");
        assert_eq!(r1.to_string(), "OK");
        assert_eq!(r2.to_string(), "OK|hello");
    }

    #[test]
    fn server_response_format_err() {
        let r1 = ServerResponse::err("");
        let r2 = ServerResponse::err("bad");
        assert_eq!(r1.to_string(), "ERROR");
        assert_eq!(r2.to_string(), "ERROR|bad");
    }

    #[test]
    fn stream_command_all_is_valid() {
        let (tx, rx) = unbounded();
        drop(tx);

        let cmd = Command::Stream;
        let tcp_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1234);

        let parts = vec!["udp://127.0.0.1:34254".into(), "ALL".into()];
        let client = cmd.make_client(1, tcp_addr, rx, parts);

        assert!(client.is_ok());
    }

    #[test]
    fn stream_command_rejects_bad_udp_scheme() {
        let (tx, rx) = unbounded();
        drop(tx);

        let cmd = Command::Stream;
        let tcp_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1234);

        let parts = vec!["http://127.0.0.1:34254".into(), "ALL".into()];
        let client = cmd.make_client(1, tcp_addr, rx, parts);

        assert!(client.is_err());
    }
}
