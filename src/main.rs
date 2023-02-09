use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.coincap.io/v2";
const START_AND_END: &str = "start=1356931594000&end=1675817253000";

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

async fn get_coin_data(url: &str, name: &str, interval: &str) -> Result<CoinData, reqwest::Error> {
    let url = format!(
        "{}/assets/{}/history?interval={}&{}",
        url, name, interval, START_AND_END
    );
    println!("url: {}", url);
    let resp = reqwest::get(&url).await?.json::<CoinData>().await?;
    println!("resp");
    Ok(resp)
}

async fn get_coins(url: &str) -> Result<CryptoList, reqwest::Error> {
    let url = format!("{}/assets", url).to_string();
    let resp = reqwest::get(&url).await?.json::<CryptoList>().await?;
    Ok(resp)
}

async fn get_coin_info(
    coin_data: CoinData,
    name: &str,
) -> Result<CoinInfo, Box<dyn std::error::Error>> {
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
    let coin_list = get_coins(API_URL).await;
    match coin_list {
        Ok(coin_list) => {
            for coin in coin_list.data {
                let coin_data = get_coin_data(API_URL, &coin.id, "d1").await;
                match coin_data {
                    Ok(data) => {
                        let coin_info = get_coin_info(data, &coin.id).await;
                        match coin_info {
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
                    Err(e) => println!("Error: {}", e),
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::GET;
    use httpmock::MockServer;

    #[tokio::test]
    async fn test_get_coins() {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(GET).path("/v2/assets");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                "data": [
                    {
                        "id": "bitcoin",
                        "rank": "1",
                        "symbol": "BTC",
                        "name": "Bitcoin"
                    },
                    {
                        "id": "ethereum",
                        "rank": "2",
                        "symbol": "ETH",
                        "name": "Ethereum"
                    }
                ]
            }"#,
                );
        });
        let coin_list = get_coins(&mock_server.url("/v2")).await;
        assert!(coin_list.is_ok());
        let coin_list = coin_list.unwrap();
        assert_eq!(coin_list.data.len(), 2);
        assert_eq!(coin_list.data[0].id, "bitcoin");
        assert_eq!(coin_list.data[0].rank, "1");
        assert_eq!(coin_list.data[0].symbol, "BTC");
        assert_eq!(coin_list.data[0].name, "Bitcoin");
        assert_eq!(coin_list.data[1].id, "ethereum");
        assert_eq!(coin_list.data[1].rank, "2");
        assert_eq!(coin_list.data[1].symbol, "ETH");
        assert_eq!(coin_list.data[1].name, "Ethereum");
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_coin_data() {
        let url_path = format!("/v2/assets/bitcoin/history?interval=d1&{}", START_AND_END);
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(GET).path(&url_path);
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                "data": [
                    {
                        "priceUsd": "13.8"
                        "time": 1356998400000,
                    },
                    {
                        "priceUsd": "13.98"
                        "time": 1357084800000,
                    }
                ]
            }"#,
                );
        });
        println!("{}", mock_server.url(&url_path));
        println!("{}", mock_server.url("/v2"));
        let coin_data = get_coin_data(&mock_server.url("/v2"), "bitcoin", "d1").await;
        assert!(coin_data.is_ok());
        let coin_data = coin_data.unwrap();
        assert_eq!(coin_data.data.len(), 2);
        assert_eq!(coin_data.data[0].time, 1356998400000);
        assert_eq!(coin_data.data[0].priceUsd, "13.8");
        assert_eq!(coin_data.data[1].time, 1357084800000);
        assert_eq!(coin_data.data[1].priceUsd, "13.98");
        mock.assert();
    }

    #[tokio::test]
    // write a test for the function get_coin_info which doesn't use a mock server as the function doesn't need to make any HTTP requests
    async fn test_get_coin_info() {
        // mock a CoinData struct
        let coin_data = CoinData {
            data: vec![
                PriceData {
                    time: 1356998400000,
                    priceUsd: "13.8".to_string(),
                },
                PriceData {
                    time: 1357084800000,
                    priceUsd: "13.98".to_string(),
                },
                PriceData {
                    time: 1357084800000,
                    priceUsd: "13.9".to_string(),
                },
            ],
        };
        let coin_info = get_coin_info(coin_data, "bitcoin").await;
        assert!(coin_info.is_ok());
        let coin_info = coin_info.unwrap();
        assert_eq!(coin_info.name, "bitcoin");
        assert_eq!(coin_info.all_time_high, 13.98);
        assert_eq!(coin_info.all_time_low, 13.8);
        assert_eq!(coin_info.current_price, 13.9);
    }
}
