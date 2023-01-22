# Financial Analysis - Stock Value Investing Tool

A rust test project for analyzing stocks and identifying value investment opportunities based on some financial metrics.

## Setup

Install dependencies:
```bash
cargo build
```

Configure the application by editing `config/example.json`:
   - Add your Finnhub API key to the `data_fetching.finnhub_api_key` field
   - Adjust analysis parameters in the `analysis` section as needed

Example configuration:
```json
{
    "data_fetching": {
        "finnhub_api_key": "insert api key here",
        "max_api_calls_per_minute": 30,
        "considered_exchanges": [
            "US"
        ]
    },
    "analysis": {
        "pe_limits": [
            2.0,
            22.5
        ],
        "pb_limits": [
            0.4,
            5.0
        ],
        "earnings_growth_5y_min": 6.0,
        "dividend_per_share_min": 0.1,
        "dividend_growth_5y_min": 5.0,
        "current_ratio_min": 1.5,
        "debt_equity_max": 2.0,
        "market_cap_min": 20e3,
        "nor_aaa_10y_bond_yield": 0.0295,
        "us_aaa_10y_bond_yield": 0.0336
    }
}
```

## Running the Application

Run the main program:
```bash
cargo run config/example.json
```

## Running Tests

Run tests with output:
```bash
cargo test -- --nocapture
```

