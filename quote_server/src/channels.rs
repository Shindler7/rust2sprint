//! Каналы трансляции данных и поддерживающие методы.

use crate::config::{CHANNEL_TIMEOUT_MS, GEN_TICKERS_DURATION_MS};
use crate::generator::QuoteGenerator;
use crate::models::ClientManager;
use crossbeam_channel::{Receiver, SendTimeoutError, Sender};
use log::{error, info, warn};
use std::sync::atomic::AtomicBool;
use std::{
    sync::atomic::Ordering,
    sync::{Arc, Mutex},
    thread,
    thread::JoinHandle,
    time::Duration,
};

/// Запустить ленту котировок.
pub fn start_generator(tx: Sender<String>) -> JoinHandle<()> {
    let mut generator = QuoteGenerator::new().unwrap_or_else(|err| {
        error!("Создать генератор не удалось: {}", err);
        panic!("ошибка генератора: {err}")
    });

    thread::spawn(move || {
        info!("Генератор котировок запущен");

        loop {
            thread::sleep(Duration::from_millis(GEN_TICKERS_DURATION_MS));

            if let Ok(quote) = generator.next_gen() {
                let quote_json = match serde_json::to_string(&quote) {
                    Ok(json) => json,
                    Err(err) => {
                        warn!("Ошибка преобразования тикера {quote} в json: {err}");
                        continue;
                    }
                };
                match tx.send_timeout(quote_json, Duration::from_millis(GEN_TICKERS_DURATION_MS)) {
                    Ok(_) => (),
                    Err(SendTimeoutError::Timeout(_)) => {
                        warn!("Канал котировок занят (timeout)");
                    }
                    Err(SendTimeoutError::Disconnected(_)) => {
                        warn!("Канал котировок закрыт");
                        break;
                    }
                }
            }
        }

        info!("Генератор котировок остановлен");
    })
}

/// Диспетчер-генератор подписчиков на канал генерации тикеров.
///
/// ## Args
///
/// - `main_receiver` — основной канал-отправитель данных
/// - `clients` — экземпляр [`ClientManager`] с данными о клиентах
/// - `stop` — прерывание работы диспетчера внешней командой
pub fn gen_tickers_dispatcher(
    main_receiver: Receiver<String>,
    clients: Arc<Mutex<ClientManager>>,
    stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        loop {
            if stop.load(Ordering::SeqCst) {
                break;
            }

            match main_receiver.recv_timeout(Duration::from_millis(CHANNEL_TIMEOUT_MS)) {
                Ok(quote) => {
                    let senders: Vec<_> = {
                        let clients = match clients.lock() {
                            Ok(c) => c,
                            Err(_) => {
                                warn!("Диспетчер тикеров: ошибка блокировки ClientManager");
                                continue;
                            }
                        };

                        clients
                            .clients
                            .iter()
                            .filter(|(_, client)| !client.stop_flag.load(Ordering::SeqCst))
                            .map(|(id_client, client)| (*id_client, client.sender.clone()))
                            .collect()
                    };

                    tickers_sender(senders, &quote);
                }

                Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => break,
            }
        }
    })
}

/// Менеджер рассылки тикеров по подписчикам.
///
/// ## Args
///
/// - `senders` — HashMap с id клиентов и отправителями активных подписчиков
/// - `message` — сообщение для рассылки
fn tickers_sender(senders: Vec<(usize, Sender<String>)>, message: &str) {
    for (id, tx) in senders {
        match tx.send_timeout(
            message.to_string(),
            Duration::from_millis(GEN_TICKERS_DURATION_MS),
        ) {
            Ok(_) => (),
            Err(SendTimeoutError::Timeout(_)) => {
                warn!("Канал котировок занят (timeout) (ошибка отправки клиенту {id})");
            }
            Err(SendTimeoutError::Disconnected(_)) => {
                error!("Канал котировок закрыт");
                break;
            }
        }
    }
}
