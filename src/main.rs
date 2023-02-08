use serde::{Deserialize, Serialize};

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
        "https://api.coincap.io/v2/assets/{}/history?interval={}&start=1406931594000&end=1675817253000",
        name, interval
    );
    let resp = reqwest::get(&url).await?.json::<CoinData>().await?;
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
    let coin_data = get_coin_info("bitcoin", "d1").await;
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

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_get_coin_data() {
        let _m = mock("GET", Matcher::Regex("/v2/assets/.*/history.*".to_owned()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
        "data": [
            {
                "priceUsd": "10000.00",
                "time": 1675817253000
            },
            {
                "priceUsd": "15000.00",
                "time": 1675817253100
            }
        ]
    }"#,
            )
            .create();
        let result = get_coin_data("bitcoin", "d1").await.unwrap();

        println!("URL: {}", _m);
        assert_eq!(result.data.len(), 2);
    }

    #[tokio::test]
    async fn test_get_coin_info() {
        let _m = mock(
            "GET",
            "/v2/assets/bitcoin/history?interval=d1&start=1406931594000&end=1675817253000",
        )
        .with_status(200)
        .with_body(
            r#"{
                "data": [
                    {
                        "priceUsd": "10000.00",
                        "time": 1675817253000
                    },
                    {
                        "priceUsd": "15000.00",
                        "time": 1675817253100
                    }
                ]
            }"#,
        )
        .create();
        let result = get_coin_info("bitcoin", "d1").await.unwrap();
        assert_eq!(result.all_time_high, 15000.00);
        assert_eq!(result.all_time_low, 10000.00);
        assert_eq!(result.current_price, 15000.00);
        assert_eq!(result.name, "bitcoin");
    }
}
