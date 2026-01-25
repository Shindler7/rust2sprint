//! Quote Server. Консольное приложение генерации котировок о ценах акций.
//! Например, для тикеров "AAPL", "GOOGL", "TSLA". Данные включают ряд
//! параметров, которые можно дополнять.

#![warn(missing_docs)]

mod data;
mod generator;
mod utils;

use commons::utils::get_timestamp;
use generator::StockQuote;
use std::str::FromStr;

fn main() {
    let quote = format!("JENN|2500|10|{}", get_timestamp());
    let obj = StockQuote::from_str(&quote).unwrap();

    println!("{}", obj);
}
