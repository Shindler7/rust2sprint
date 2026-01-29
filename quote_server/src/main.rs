//! Quote Server. Консольное приложение генерации котировок о ценах акций.
//! Например, для тикеров "AAPL", "GOOGL", "TSLA". Данные включают ряд
//! параметров, которые можно дополнять.

#![warn(missing_docs)]

use crate::generator::QuoteGenerator;
use std::thread::sleep;
use std::time::Duration;

mod config;
mod generator;
mod tcp;
mod udp;

fn main() {
    loop {
        let generator = QuoteGenerator::generate().unwrap();
        // if generator.quote.ticker.eq("GOOGL") {
        //     println!("{}", generator.quote);
        // }

        println!("{}", generator.quote);

        sleep(Duration::from_millis(100));
    }
}
