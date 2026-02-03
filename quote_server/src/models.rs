//! Модели данных для приложения.

use commons::errors::QuoteError;
use crossbeam_channel::{Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{atomic::AtomicBool, Arc};
use url::Url;

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
    /// Персональный отправитель котировок.
    pub sender: Sender<String>,
    /// Получатель котировок.
    pub recv: Receiver<String>,
    /// Флаг остановки.
    pub stop_flag: Arc<AtomicBool>,
}

impl ClientSubscription {
    /// Создать нового клиента с указанными параметрами.
    ///
    /// - `unique_id` — уникальный идентификатор клиента в сессии
    /// - `tcp_addr` — TCP адрес клиента
    /// - `udp_url` — UDP-ссылка клиента
    /// - `tickers` — набор тикеров для подписки на обновления
    /// - `sender` — канал для отправки сообщений клиенту
    /// - `recv` — канал для получения сообщений от клиента
    pub fn new(
        unique_id: usize,
        tcp_addr: SocketAddr,
        udp_url: Url,
        tickers: HashSet<String>,
        sender: Sender<String>,
        recv: Receiver<String>,
    ) -> Self {
        let stop_flag = Arc::new(AtomicBool::new(false));
        Self {
            unique_id,
            tcp_addr,
            udp_url,
            tickers,
            sender,
            recv,
            stop_flag,
        }
    }
}

/// Менеджер клиентов.
#[derive(Debug, Default)]
pub struct ClientManager {
    /// `HashMap` активных клиентов, где ключ — уникальный id сессии.
    pub clients: HashMap<usize, ClientSubscription>,
}

impl ClientManager {
    /// Создать менеджера.
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// Проверить, существует ли клиент с предоставленным id.
    pub fn id_exists(&self, unique_id: usize) -> bool {
        self.clients.contains_key(&unique_id)
    }

    /// Добавить нового клиента.
    pub fn add_client(&mut self, client: ClientSubscription) -> Result<(), QuoteError> {
        if self.id_exists(client.unique_id) {
            return Err(QuoteError::value_err("Клиент уже существует"));
        }
        self.clients.insert(client.unique_id, client);
        Ok(())
    }

    /// Удалить клиента.
    pub fn remove_client(&mut self, unique_id: usize) -> Result<ClientSubscription, QuoteError> {
        self.clients
            .remove(&unique_id)
            .ok_or_else(|| QuoteError::command_err("задачи отсутствуют"))
    }
}
