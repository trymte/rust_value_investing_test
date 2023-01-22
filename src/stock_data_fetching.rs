use anyhow::{Error, Result};
use config::Config;
use reqwest::Url;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct DataFetchConfig {
    pub finnhub_api_key: String,
    pub max_api_calls_per_minute: u32,
    pub considered_exchanges: Vec<String>,
}

impl DataFetchConfig {
    pub fn from_json_value(json: serde_json::Value) -> Self {
        
        serde_json::from_value(json).unwrap()
    }

    pub fn from_file(filename: &str) -> Self {
        
        Config::builder()
            .add_source(config::File::with_name(filename))
            .build()
            .unwrap()
            .try_deserialize::<DataFetchConfig>()
            .unwrap()
    }

    pub fn to_file(&self, filename: &str) {
        let serialized_cfg = serde_json::to_string(&self).unwrap();
        println!("{serialized_cfg}");
        serde_json::to_writer_pretty(std::fs::File::create(filename).unwrap(), &self).unwrap();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StockInfo {
    pub symbol: String,
    pub currency: String,
    pub description: String,
}

pub async fn extract_stock_list_from_exchange(
    client: &reqwest::Client,
    exchange: &String,
    api_key: &String,
) -> Result<Vec<StockInfo>, Error> {
    let mut stocks: Vec<StockInfo> = Vec::new();
    let url = format!(
        "https://finnhub.io/api/v1/stock/symbol?exchange={exchange}&token={api_key}"
    );
    let url = Url::parse(&url)?;
    let response = client
        .get(url)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    // println!("{:?}", response.as_array().unwrap());
    for stock_info_val in response.as_array().unwrap() {
        // println!("{}", stock_info_val.as_str().unwrap());

        if stock_info_val["type"].as_str().unwrap() != "Common Stock" {
            continue;
        }

        let stock_info = StockInfo {
            symbol: stock_info_val["symbol"].as_str().unwrap().to_string(),
            currency: stock_info_val["currency"].as_str().unwrap().to_string(),
            description: stock_info_val["description"].as_str().unwrap().to_string(),
        };
        stocks.push(stock_info);
    }

    Ok(stocks)
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CompanyQuote {
    pub c: f64,  // current price
    pub h: f64,  // high price
    pub l: f64,  // low price
    pub o: f64,  // open price
    pub pc: f64, // previous close price
    pub t: i128, // timestamp
}

impl CompanyQuote {
    pub async fn get(
        client: &reqwest::Client,
        symbol: &String,
        api_key: &String,
    ) -> Result<Self, Error> {
        let url = format!(
            "https://finnhub.io/api/v1/quote?symbol={symbol}&token={api_key}"
        );
        let url = Url::parse(&url)?;
        let res = client.get(url).send().await?.json::<CompanyQuote>().await?;
        Ok(res)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CompanyInformation {
    pub name: String,
    pub ticker: String,
    pub exchange: String,
    pub currency: String,
    pub country: String,
    pub industry: String,
    pub market_cap: f64,         // In Million USD unless otherwise specified
    pub shares_outstanding: f64, // In Millions unless otherwise specified
    pub ipo: String,
    pub weburl: String,
    pub finnhub_industry: String,
}

impl CompanyInformation {
    pub async fn get(
        client: &reqwest::Client,
        symbol: &String,
        api_key: &String,
    ) -> Result<Self, Error> {
        let url = format!(
            "https://finnhub.io/api/v1/stock/profile2?symbol={symbol}&token={api_key}"
        );
        let url = Url::parse(&url)?;
        let response = client
            .get(url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        //println!("{:?}", response);
        let res = CompanyInformation {
            name: response["name"].as_str().unwrap().to_string(),
            ticker: response["ticker"].as_str().unwrap().to_string(),
            exchange: response["exchange"].as_str().unwrap().to_string(),
            currency: response["currency"].as_str().unwrap().to_string(),
            country: response["country"].as_str().unwrap().to_string(),
            industry: response["finnhubIndustry"].as_str().unwrap().to_string(),
            market_cap: response["marketCapitalization"].as_f64().unwrap(),
            shares_outstanding: response["shareOutstanding"].as_f64().unwrap(),
            ipo: response["ipo"].as_str().unwrap().to_string(),
            weburl: response["weburl"].as_str().unwrap().to_string(),
            finnhub_industry: response["finnhubIndustry"].as_str().unwrap().to_string(),
        };
        //println!("{:?}", res);
        Ok(res)
    }
}

// The financials are annual unless otherwise specified
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CompanyFinancials {
    pub pb_ratio: Option<f64>,
    pub ps_ratio: Option<f64>,
    pub pe_ratio: Option<f64>,
    pub dividend_per_share: Option<f64>,
    pub dividend_per_share_5_yr_avg: Option<f64>,
    pub dividend_growth_5_yr_avg: Option<f64>,
    pub earnings_per_share: Option<f64>,
    pub earnings_growth: Option<f64>,
    pub earnings_growth_5_yr_avg: Option<f64>,
    pub book_value_per_share: Option<f64>,
    pub tangible_book_value_per_share: Option<f64>,
    pub total_debt_to_total_equity: Option<f64>,
    pub long_term_debt_to_equity: Option<f64>,
    pub current_ratio: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub return_on_avg_equity: Option<f64>,
    pub return_on_avg_equity_5_yr: Option<f64>,
    pub return_on_avg_assets_5_yr: Option<f64>,
    pub return_on_investments: Option<f64>,
    pub return_on_investments_5_yr: Option<f64>,
    pub net_profit_margin: Option<f64>,
    pub net_profit_margin_5_yr_avg: Option<f64>,
    pub net_profit_margin_growth_5_yr_avg: Option<f64>,
    pub total_current_assets: Option<f64>,
    pub total_current_liabilities: Option<f64>,
    pub total_current_long_term_debt: Option<f64>,
}

impl CompanyFinancials {
    pub async fn get(
        client: &reqwest::Client,
        symbol: &String,
        api_key: &String,
    ) -> Result<Self, Error> {
        let url = format!(
            "https://finnhub.io/api/v1/stock/metric?symbol={symbol}&metric=all&token={api_key}"
        );

        let url = Url::parse(&url)?;
        let financial_response = client
            .get(url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let url = format!(
            "https://finnhub.io/api/v1/stock/financials-reported?symbol={symbol}&token={api_key}&freq=annual"
        );
        let url = Url::parse(&url)?;
        let balance_sheet_response = client
            .get(url)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Self::from_serde_json_value(financial_response, balance_sheet_response)
    }

    pub fn from_serde_json_value(
        financial_response: serde_json::Value,
        bs_response: serde_json::Value,
    ) -> Result<Self, Error> {
        if bs_response["data"][0]["report"]["bs"].is_null() {
            return Err(Error::msg("Null value for company balance sheet."));
        }
        let mut total_current_assets = Some(0.0_f64);
        let mut total_current_liabilities = Some(0.0_f64);
        let mut total_current_long_term_debt = Some(0.0_f64);
        //println!("Financials: {:?}", financial_response["metric"]);
        for entry in bs_response["data"][0]["report"]["bs"].as_array().unwrap() {
            //println!("Entry: {:?}", entry);
            let label = entry["label"].as_str().unwrap();
            if label == "Total current assets" {
                total_current_assets = Some(entry["value"].as_f64().unwrap() / 1e6_f64);
            }

            if label == "Total current liabilities" {
                total_current_liabilities = Some(entry["value"].as_f64().unwrap() / 1e6_f64);
            }

            let concept = entry["concept"].as_str().unwrap();
            if label == "Term debt" && concept == "us-gaap_LongTermDebtCurrent" {
                total_current_long_term_debt = Some(entry["value"].as_f64().unwrap() / 1e6_f64);
            }
        }

        let mut res = CompanyFinancials {
            pb_ratio: financial_response["metric"]["pbAnnual"].as_f64(),
            ps_ratio: financial_response["metric"]["psAnnual"].as_f64(),
            pe_ratio: financial_response["metric"]["peNormalizedAnnual"].as_f64(),
            dividend_per_share: financial_response["metric"]["dividendPerShareAnnual"].as_f64(),
            dividend_per_share_5_yr_avg: financial_response["metric"]["dividendPerShare5Y"]
                .as_f64(),
            dividend_growth_5_yr_avg: financial_response["metric"]["dividendGrowthRate5Y"].as_f64(),
            earnings_per_share: financial_response["metric"]["epsNormalizedAnnual"].as_f64(),
            earnings_growth: financial_response["metric"]["epsGrowth"].as_f64(),
            earnings_growth_5_yr_avg: financial_response["metric"]["epsGrowth5Y"].as_f64(),
            book_value_per_share: financial_response["metric"]["bookValuePerShare"].as_f64(),
            tangible_book_value_per_share: financial_response["metric"]
                ["tangibleBookValuePerShareAnnual"]
                .as_f64(),
            total_debt_to_total_equity: financial_response["metric"]["totalDebt/totalEquityAnnual"]
                .as_f64(),
            long_term_debt_to_equity: financial_response["metric"]["longTermDebt/equityAnnual"]
                .as_f64(),
            current_ratio: financial_response["metric"]["currentRatioAnnual"].as_f64(),
            quick_ratio: financial_response["metric"]["quickRatioAnnual"].as_f64(),
            return_on_avg_equity: financial_response["metric"]["roeAnnual"].as_f64(),
            return_on_avg_equity_5_yr: financial_response["metric"]["roae5Y"].as_f64(),
            return_on_avg_assets_5_yr: financial_response["metric"]["roaa5Y"].as_f64(),
            return_on_investments: financial_response["metric"]["roiAnnual"].as_f64(),
            return_on_investments_5_yr: financial_response["metric"]["roi5Y"].as_f64(),
            net_profit_margin: financial_response["metric"]["netProfitMarginAnnual"].as_f64(),
            net_profit_margin_5_yr_avg: financial_response["metric"]["netProfitMargin5Y"].as_f64(),
            net_profit_margin_growth_5_yr_avg: financial_response["metric"]["netMarginGrowth5Y"]
                .as_f64(),
            total_current_assets,
            total_current_liabilities,
            total_current_long_term_debt,
        };

        res.total_debt_to_total_equity = res.total_debt_to_total_equity.map(|x| x / 100.0);
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_stock_list_fetching() -> Result<()> {
        let config_json_value = Config::builder()
            .add_source(config::File::with_name("config/example.json"))
            .build()
            .unwrap()
            .try_deserialize::<serde_json::Value>()
            .unwrap();

        let cfg = DataFetchConfig::from_json_value(config_json_value["data_fetching"].clone());

        let client = reqwest::Client::new();
        let considered_exchanges = cfg.considered_exchanges.clone();

        let mut exchange_stock_list: HashMap<String, Vec<StockInfo>> = HashMap::new();
        for exchange in considered_exchanges {
            // println!("Exchange: {}", exchange);
            let stock_list =
                extract_stock_list_from_exchange(&client, &exchange, &cfg.finnhub_api_key)
                    .await
                    .unwrap();

            // println!("{} Stock list: {:?}", exchange, stock_list);
            exchange_stock_list.insert(exchange, stock_list);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_company_quote() -> Result<()> {
        let config_json_value = Config::builder()
            .add_source(config::File::with_name("config/example.json"))
            .build()
            .unwrap()
            .try_deserialize::<serde_json::Value>()
            .unwrap();

        let my_config =
            DataFetchConfig::from_json_value(config_json_value["data_fetching"].clone());
        println!("{:?}", my_config);

        let symbol: String = "AAPL".to_string();
        let client = reqwest::Client::new();

        let res = CompanyQuote::get(&client, &symbol, &my_config.finnhub_api_key)
            .await
            .unwrap();
        println!("{}'s current stock price: {}", symbol, res.c);
        println!("{}'s company quote: {:?}", symbol, res);

        Ok(())
    }

    #[tokio::test]
    async fn test_company_information() -> Result<()> {
        let config_json_value = Config::builder()
            .add_source(config::File::with_name("config/example.json"))
            .build()
            .unwrap()
            .try_deserialize::<serde_json::Value>()
            .unwrap();

        let my_config =
            DataFetchConfig::from_json_value(config_json_value["data_fetching"].clone());

        let symbol: String = "AAPL".to_string();
        let client = reqwest::Client::new();
        let res = CompanyInformation::get(&client, &symbol, &my_config.finnhub_api_key).await;
        println!("{}'s company information: {:?}", symbol, res);

        Ok(())
    }

    #[tokio::test]
    async fn test_company_financials() -> Result<()> {
        let config_json_value = Config::builder()
            .add_source(config::File::with_name("config/example.json"))
            .build()
            .unwrap()
            .try_deserialize::<serde_json::Value>()
            .unwrap();

        let data_fetch_config =
            DataFetchConfig::from_json_value(config_json_value["data_fetching"].clone());

        let symbol: String = "AAPL".to_string();
        let client = reqwest::Client::new();
        let res =
            CompanyFinancials::get(&client, &symbol, &data_fetch_config.finnhub_api_key).await;
        println!("{}'s company financials: {:?}", symbol, res);

        Ok(())
    }
}
