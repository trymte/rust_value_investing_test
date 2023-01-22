use crate::stock_data_fetching::{
    extract_stock_list_from_exchange, CompanyFinancials, CompanyInformation, CompanyQuote,
    DataFetchConfig, StockInfo,
};
use anyhow::{Error, Result};
use config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tokio::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
pub struct AnalysisConfig {
    pub pe_limits: [f64; 2],         // ratio
    pub pb_limits: [f64; 2],         // ratio
    pub earnings_growth_5y_min: f64, // In percent / 100
    pub dividend_per_share_min: f64, // In dollars
    pub dividend_growth_5y_min: f64, // In percent / 100
    pub current_ratio_min: f64,      // ratio
    pub debt_equity_max: f64,        // ratio
    pub market_cap_min: f64,         // In millions
    pub nor_aaa_10y_bond_yield: f64, // In percent / 100
    pub us_aaa_10y_bond_yield: f64,  // In percent / 100
}

impl AnalysisConfig {
    pub fn from_json_value(json: serde_json::Value) -> Self {
        
        serde_json::from_value(json).unwrap()
    }

    pub fn from_file(filename: &str) -> Self {
        
        Config::builder()
            .add_source(config::File::with_name(filename))
            .build()
            .unwrap()
            .try_deserialize::<AnalysisConfig>()
            .unwrap()
    }

    pub fn to_file(&self, filename: &str) {
        let serialized_cfg = serde_json::to_string(&self).unwrap();
        println!("{serialized_cfg}");
        serde_json::to_writer_pretty(std::fs::File::create(filename).unwrap(), &self).unwrap();
    }
}

pub fn check_pe(financials: &CompanyFinancials, analysis_config: &AnalysisConfig) -> Result<bool> {
    let pe = financials.pe_ratio.unwrap_or(-1e12);
    if pe < -1e11 {
        return Err(Error::msg("No price/earnings ratio"));
    }
    println!(
        "Price/earnings: {:.2} | Limits: {:?}",
        pe, analysis_config.pe_limits
    );
    Ok(pe >= analysis_config.pe_limits[0] && pe <= analysis_config.pe_limits[1])
}

pub fn check_dividends(
    financials: &CompanyFinancials,
    analysis_config: &AnalysisConfig,
) -> Result<bool> {
    let dividend_per_share = financials.dividend_per_share.unwrap_or(-1e12);
    let dividend_per_share_5_yr_avg = financials.dividend_per_share_5_yr_avg.unwrap_or(-1e12);
    let dividend_growth_5_yr_avg = financials.dividend_growth_5_yr_avg.unwrap_or(-1e12);
    if dividend_per_share < -1e11
        || dividend_per_share_5_yr_avg < -1e11
        || dividend_growth_5_yr_avg < -1e11
    {
        return Err(Error::msg("No dividends"));
    }

    println!(
        "Dividend per share: {:.2} | Dividend per share 5yr avg: {:.2} | Dividend growth 5y: {:.2} | Minimum 5y: {:.2}",
        dividend_per_share, dividend_per_share_5_yr_avg, dividend_growth_5_yr_avg, analysis_config.dividend_growth_5y_min
    );
    Ok(dividend_per_share >= analysis_config.dividend_per_share_min
        && dividend_per_share_5_yr_avg >= analysis_config.dividend_per_share_min
        && dividend_growth_5_yr_avg >= analysis_config.dividend_growth_5y_min)
}

pub fn check_earnings_growth(
    financials: &CompanyFinancials,
    analysis_config: &AnalysisConfig,
) -> Result<bool> {
    let earnings_growth = financials.earnings_growth.unwrap_or(-1e12);
    let earnings_growth_5_yr_avg = financials.earnings_growth_5_yr_avg.unwrap_or(-1e12);
    if earnings_growth < -1e11 || earnings_growth_5_yr_avg < -1e11 {
        return Err(Error::msg("No earnings growth"));
    }

    println!(
        "Earnings growth: {:.2} | Earnings growth 5y: {:.2} | Minimum 5y: {:.2}",
        earnings_growth, earnings_growth_5_yr_avg, analysis_config.earnings_growth_5y_min
    );
    Ok(
        earnings_growth >= 0.0
            && earnings_growth_5_yr_avg >= analysis_config.earnings_growth_5y_min,
    )
}

pub fn check_pb(
    financials: &CompanyFinancials,
    industry: &String,
    analysis_config: &AnalysisConfig,
) -> Result<bool> {
    let pb = financials.pb_ratio.unwrap_or(-1e12);
    if pb < -1e11 {
        return Err(Error::msg("No price/book ratio"));
    }
    let mut limits = analysis_config.pb_limits;
    if industry == "Technology" {
        limits[1] *= 5.0;
    }

    println!(
        "Price/book: {:.2} | Limits: {:?}",
        pb, analysis_config.pb_limits
    );
    Ok(pb >= analysis_config.pb_limits[0] && pb <= analysis_config.pb_limits[1])
}

pub fn check_debt_equity(
    financials: &CompanyFinancials,
    analysis_config: &AnalysisConfig,
) -> Result<bool> {
    let debt_equity = financials.total_debt_to_total_equity.unwrap_or(-1e12);
    if debt_equity < -1e11 {
        return Err(Error::msg("No debt/equity ratio"));
    }
    println!(
        "Debt/equity: {:.2} | Maximum: {:.2}",
        debt_equity, analysis_config.debt_equity_max
    );
    Ok(debt_equity <= analysis_config.debt_equity_max)
}

pub fn check_working_capital(
    financials: &CompanyFinancials,
    information: &CompanyInformation,
    quote: &CompanyQuote,
) -> Result<bool> {
    let total_current_assets = financials.total_current_assets.unwrap_or(-1e12);
    let total_current_liabilities: f64 = financials.total_current_liabilities.unwrap_or(-1e12);
    let total_current_long_term_debt: f64 =
        financials.total_current_long_term_debt.unwrap_or(-1e12);
    if total_current_assets < -1e11
        || total_current_liabilities < -1e11
        || total_current_long_term_debt < -1e11
    {
        return Err(Error::msg(
            "No current assets, liabilities or long term debt",
        ));
    }
    let n_stocks = information.shares_outstanding;
    let working_capital_per_share = (total_current_assets - total_current_liabilities) / n_stocks;
    let total_current_long_term_debt_per_share = total_current_long_term_debt / n_stocks;
    println!(
        "Working capital per share: {:.2} | Total debt per share: {:.2} | Current price: {:.2}",
        working_capital_per_share, total_current_long_term_debt_per_share, quote.c
    );

    Ok(0.0 <= working_capital_per_share && working_capital_per_share >= 2.0 * quote.c / 3.0)
}

// total debt <= 1.1 * working capital
// price < 1.2 * net_tangible_assets_per_share
// working capital per share > price per share => bra
// asset values per share >= 2/3 * price per share => bra

pub fn check_market_cap(
    information: &CompanyInformation,
    analysis_config: &AnalysisConfig,
) -> bool {
    let market_cap = information.market_cap;
    println!(
        "Market cap: {:.2} | minimum: {:.2}",
        market_cap, analysis_config.market_cap_min
    );
    market_cap >= analysis_config.market_cap_min
}

pub fn check_current_ratio(
    financials: &CompanyFinancials,
    analysis_config: &AnalysisConfig,
) -> Result<bool> {
    let current_ratio = financials.current_ratio.unwrap_or(-1e12);
    if current_ratio < -1e11 {
        return Err(Error::msg("No current ratio"));
    }
    println!(
        "Current ratio: {:.2} | Minimum: {:.2}",
        current_ratio, analysis_config.current_ratio_min
    );
    Ok(current_ratio >= analysis_config.current_ratio_min)
}

pub struct StockAnalyzer {
    pub data_fetch_config: DataFetchConfig,
    pub analysis_config: AnalysisConfig,
    pub client: Client,
    num_api_calls: u32,
}

impl StockAnalyzer {
    pub fn new(settings_filename: &str) -> Self {
        let config_json_value = Config::builder()
            .add_source(config::File::with_name(settings_filename))
            .build()
            .unwrap()
            .try_deserialize::<serde_json::Value>()
            .unwrap();
        let data_fetch_config =
            DataFetchConfig::from_json_value(config_json_value["data_fetching"].clone());
        let analysis_config =
            AnalysisConfig::from_json_value(config_json_value["analysis"].clone());

        let client = reqwest::Client::new();
        let num_api_calls = 0_u32;
        Self {
            data_fetch_config,
            analysis_config,
            client,
            num_api_calls,
        }
    }

    async fn update_api_calls(&mut self) {
        self.num_api_calls += 1;
        //println!("Number of API calls: {}", self.num_api_calls);
        if self.num_api_calls >= self.data_fetch_config.max_api_calls_per_minute {
            println!("Reached API call limit, sleeping for 30+ seconds...");
            sleep(Duration::from_secs(31)).await;
            self.num_api_calls = 0;
        }
    }

    pub async fn get_exchange_stock_list(&mut self, exchange: &String) -> Result<Vec<StockInfo>> {
        self.update_api_calls().await;
        let stock_list = extract_stock_list_from_exchange(
            &self.client,
            exchange,
            &self.data_fetch_config.finnhub_api_key,
        )
        .await
        .expect("No stock list");

        Ok(stock_list)
    }

    pub async fn get_stock_data(
        &mut self,
        stock_info: &StockInfo,
    ) -> Result<(CompanyFinancials, CompanyInformation, CompanyQuote)> {
        self.update_api_calls().await;
        //println!("{}: Started getting financials...", stock_info.symbol);
        let financials = CompanyFinancials::get(
            &self.client,
            &stock_info.symbol,
            &self.data_fetch_config.finnhub_api_key,
        )
        .await?;
        //println!("{}: Finished getting financials", stock_info.symbol);

        self.update_api_calls().await;
        //println!("{}: Started getting information...", stock_info.symbol);
        let information = CompanyInformation::get(
            &self.client,
            &stock_info.symbol,
            &self.data_fetch_config.finnhub_api_key,
        )
        .await?;
        //println!("{}: Finished getting information", stock_info.symbol);

        self.update_api_calls().await;
        //println!("{} Started getting the quote..", stock_info.symbol);
        let quote: CompanyQuote = CompanyQuote::get(
            &self.client,
            &stock_info.symbol,
            &self.data_fetch_config.finnhub_api_key,
        )
        .await?;
        //println!("{}: Finished getting the quote", stock_info.symbol);

        Ok((financials, information, quote))
    }

    pub async fn check_stock(&mut self, stock_info: &StockInfo) -> Result<bool> {
        let (financials, information, quote) = self.get_stock_data(stock_info).await?;

        println!("{}: Started check...", stock_info.symbol);

        let industry = information.industry.clone();
        let market_cap_good = check_market_cap(&information, &self.analysis_config);
        if market_cap_good {
            println!("{}: MARKET CAP METRICS SATISFIED", stock_info.symbol);
        } else {
            return Err(Error::msg("Not big enough company"));
        }

        //println!("Num api calls: {}", self.num_api_calls);
        let pe_good = check_pe(&financials, &self.analysis_config).unwrap_or_else(|_error| {
            println!("{}: No price/earnings ratio.", stock_info.symbol);
            false
        });
        if pe_good {
            println!("{}: P/E METRICS SATISFIED", stock_info.symbol);
        } else {
            return Err(Error::msg("Not good enough P/E ratio"));
        }

        let dividend_good =
            check_dividends(&financials, &self.analysis_config).unwrap_or_else(|_error| {
                println!("{}: No dividend.", stock_info.symbol);
                false
            });
        if dividend_good {
            println!("{}: DIVIDEND METRICS SATISFIED", stock_info.symbol);
        } else {
            return Err(Error::msg("Not good enough dividends"));
        }

        let earnings_growth_good = check_earnings_growth(&financials, &self.analysis_config)
            .unwrap_or_else(|_error| {
                println!("{}: No earnings growth.", stock_info.symbol);
                false
            });
        if earnings_growth_good {
            println!("{}: EARNINGS GROWTH METRICS SATISFIED", stock_info.symbol);
        }

        let pb_good =
            check_pb(&financials, &industry, &self.analysis_config).unwrap_or_else(|_error| {
                println!("{}: No price/book ratio.", stock_info.symbol);
                false
            });
        if pb_good {
            println!("{}: P/B METRICS SATISFIED", stock_info.symbol);
        }

        let debt_equity_good = check_debt_equity(&financials, &self.analysis_config)
            .unwrap_or_else(|_error| {
                println!("{}: No debt/equity ratio.", stock_info.symbol);
                false
            });
        if debt_equity_good {
            println!("{}: DEBT/EQUITY METRICS SATISFIED", stock_info.symbol);
        }

        let working_capital_good = check_working_capital(&financials, &information, &quote)
            .unwrap_or_else(|_error| {
                println!("{}: No working capital.", stock_info.symbol);
                false
            });
        if working_capital_good {
            println!("{}: WORKING CAPITAL METRICS SATISFIED", stock_info.symbol);
        }
        println!("Finished checking {}...", stock_info.symbol);

        let total_check = pe_good && pb_good && debt_equity_good && working_capital_good;
        if total_check {
            println!(
                "{}: TOTAL CHECK SATISFIED: {}",
                stock_info.symbol, total_check
            )
        }

        Ok(total_check)
    }
}

mod tests {
    use crate::stock_data_fetching::StockInfo;
    use crate::financial_analysis::StockAnalyzer;
    #[tokio::test]
    async fn test_stock_analysis() -> anyhow::Result<()> {
        let settings_filename = "config/settings";
        let mut stock_analyzer = StockAnalyzer::new(settings_filename);

        let stock_info = StockInfo {
            symbol: "AAPL".to_string(),
            currency: "USD".to_string(),
            description: "Apple".to_string(),
        };

        let is_good = match stock_analyzer.check_stock(&stock_info).await {
            Ok(is_good) => is_good,
            Err(error) => {
                println!("Error: {}", error);
                false
            }
        };
        println!("{}: Is good? {}", stock_info.symbol, is_good);

        Ok(())
    }
}
