use std::error::Error;

use cli_candlestick_chart::{Candle, Chart};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
struct BinanceKlinesItem {
    open_time: u64,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
    close_time: u64,
    quote_asset_volume: String,
    number_of_trades: u64,
    taker_buy_base_asset_volume: String,
    taker_buy_quote_asset_volume: String,
    ignore: String,
}

pub async fn display_cs() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let candles =
        reqwest::Client::get(&client, "https://api.binance.com/api/v1/klines?symbol=CHZUSDT&interval=1h")
            .send()
            .await.unwrap()
            .json::<Vec<BinanceKlinesItem>>().await
            .iter()
            .map(|candle| {
                Candle::new(
                    candle[0].open.parse::<f64>().unwrap(),
                    candle[0].high.parse::<f64>().unwrap(),
                    candle[0].low.parse::<f64>().unwrap(),
                    candle[0].close.parse::<f64>().unwrap(),
                    Some(candle[0].volume.parse::<f64>().unwrap()),
                    Some(candle[0].open_time as i64),
                )
            })
            .collect::<Vec<Candle>>();

    let mut chart = Chart::new(&candles);

    chart.set_name(String::from("CHZ/USDT"));
    chart.set_bull_color(1, 205, 254);
    chart.set_bear_color(255, 107, 153);
    chart.set_vol_bull_color(1, 205, 254);
    chart.set_vol_bear_color(255, 107, 153);
    chart.set_volume_pane_height(4);
    chart.set_volume_pane_enabled(true);
    // chart.set_volume_pane_unicode_fill(true);

    chart.draw();

    Ok(())
}