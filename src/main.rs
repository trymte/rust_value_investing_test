use anyhow::{Context, Result};
use financial_analysis::financial_analysis::StockAnalyzer;
use financial_analysis::stock_data_fetching::StockInfo;
use std::env;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

async fn save_stocks_to_file(stocks: Vec<StockInfo>, filename: &str) -> Result<()> {
    let mut file = File::create(filename).await?;

    for stock in stocks {
        let symbol_str = stock.symbol.as_bytes();
        file.write_all(symbol_str).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let settings_filename = env::args()
        .nth(1)
        .context("Usage: cargo run <settings_file_path>")?;

    let mut stock_analyzer = StockAnalyzer::new(&settings_filename);
    let exchange = "US".to_string();

    let stock_list = stock_analyzer.get_exchange_stock_list(&exchange).await?;
    println!("Stocks on exchange {}: {}", exchange, stock_list.len());
    
    let analyzer = Arc::new(Mutex::new(stock_analyzer));

    let results: Vec<_> = futures::future::join_all(stock_list.into_iter().map(|stock| {
        let analyzer = Arc::clone(&analyzer);
        tokio::spawn(async move {
            let mut analyzer = analyzer.lock().await;
            match analyzer.check_stock(&stock).await {
                Ok(is_good) => Ok((stock, is_good)),
                Err(e) => {
                    println!("Error checking stock {}: {}", stock.symbol, e);
                    Err(stock)
                }
            }
        })
    })).await;

    let mut worthy_stocks = Vec::new();
    let mut shitty_stocks = Vec::new();
    
    for result in results {
        match result {
            Ok(Ok((stock, true))) => worthy_stocks.push(stock),
            Ok(Ok((stock, false))) => shitty_stocks.push(stock),
            Ok(Err(stock)) => shitty_stocks.push(stock),
            Err(e) => eprintln!("Stock analysis error: {}", e),
        }
    }

    save_stocks_to_file(shitty_stocks.clone(), "shitty_stocks.txt").await?;

    println!("Worthy stocks: {worthy_stocks:?}");

    Ok(())

}