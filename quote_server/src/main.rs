//! Quote Server. Консольное приложение генерации котировок о ценах акций.
//! Например, для тикеров "AAPL", "GOOGL", "TSLA". Данные включают ряд
//! параметров, которые можно дополнять.

#![warn(missing_docs)]

mod config;
mod generator;
mod utils;

use generator::StockQuote;

fn main() {
    let obj = StockQuote::generate_new();

    println!("{}", obj);
}
