use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Cryptocurrency {
    id: String,
    rank: String,
    symbol: String,
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CryptoList {
    data: Vec<Cryptocurrency>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct PriceData {
    priceUsd: String,
    time: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct CoinData {
    data: Vec<PriceData>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CoinInfo {
    name: String,
    all_time_high: f64,
    all_time_low: f64,
    current_price: f64,
}

async fn get_coin_data(name: &str, interval: &str) -> Result<CoinData, reqwest::Error> {
    let url = format!(
        "https://api.coincap.io/v2/assets/{}/history?interval={}&start=1356931594000&end=1675817253000",
        name, interval
    );
    let resp = reqwest::get(&url).await?.json::<CoinData>().await?;
    Ok(resp)
}

async fn get_coins() -> Result<CryptoList, reqwest::Error> {
    let url = format!("https://api.coincap.io/v2/assets");
    let resp = reqwest::get(&url).await?.json::<CryptoList>().await?;
    Ok(resp)
}

async fn get_coin_info(name: &str, interval: &str) -> Result<CoinInfo, Box<dyn std::error::Error>> {
    let coin_data = get_coin_data(name, interval).await?;
    let prices = &coin_data.data;

    let all_time_high = prices
        .iter()
        .filter_map(|x| x.priceUsd.parse::<f64>().ok())
        .fold(f64::MIN, |acc, x| acc.max(x));

    let all_time_low = prices
        .iter()
        .filter_map(|x| x.priceUsd.parse::<f64>().ok())
        .fold(f64::INFINITY, |acc, x| acc.min(x));

    let current_price = match prices.last() {
        Some(last_price) => last_price
            .priceUsd
            .parse::<f64>()
            .map_err(|_| "Failed to parse priceUsd".to_owned()),
        None => Ok(0.0),
    }?;

    Ok(CoinInfo {
        name: name.to_string(),
        all_time_high,
        all_time_low,
        current_price,
    })
}

fn draw_bar_graph(upper: f64, lower: f64, current: f64, symbol: String) {
    let range = upper - lower;
    if range == 0.0 {
        println!("Upper and lower value are the same.");
        return;
    }
    let percentage = (current - lower) * 100.0 / range;
    let formatted_percentage = format!("{:.2}", percentage);
    let formatted_percentage = formatted_percentage.parse::<f64>().unwrap();
    if !(0.0..=100.0).contains(&formatted_percentage) {
        println!("Current value is not within the specified range.");
        return;
    }
    let bar = (formatted_percentage as i32) / 2;
    let formatted_percentage = format!("{:>10}", formatted_percentage);
    let padding = 50 - bar;
    println!(
        "{}|{}{}|{}",
        format_args!("{}%", formatted_percentage),
        "█".repeat(bar as usize),
        "░".repeat(padding as usize),
        symbol
    );
}

#[tokio::main]
async fn main() {
    let coin_list = get_coins().await;
    match coin_list {
        Ok(coin_list) => {
            for coin in coin_list.data {
                let coin_data = get_coin_info(&coin.id, "d1").await;
                match coin_data {
                    Ok(data) => {
                        let (upper, lower, current, name) = (
                            data.all_time_high,
                            data.all_time_low,
                            data.current_price,
                            data.name,
                        );
                        draw_bar_graph(upper, lower, current, name);
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

/// write some tests for the functions in this module
/// make the tests detailed and thorough
/// covering each possible branch of execution
/// and each possible error condition
/// use any known crates you like
/// use any testing framework you like
/// use any test runner you like
/// use any test coverage tool you like
/// use any test coverage reporting tool you like
/// use any test coverage reporting service you like



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_coin_data() {
        let coin_data = get_coin_data("bitcoin", "d1").await;
        assert!(coin_data.is_ok());
    }

    #[tokio::test]
    async fn test_get_coins() {
        let coins = get_coins().await;
        assert!(coins.is_ok());
    }

    #[tokio::test]
    async fn test_get_coin_info() {
        let coin_info = get_coin_info("bitcoin", "d1").await;
        assert!(coin_info.is_ok());
    }

    #[tokio::test]
    async fn test_draw_bar_graph() {
        draw_bar_graph(100.0, 0.0, 50.0, "BTC".to_string());
    }
}
