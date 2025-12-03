use alphafield_core::{
    ExecutionService, Order, OrderSide, OrderStatus, OrderType, QuantError, Result,
};
use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

pub struct BinanceExecutionClient {
    client: reqwest::Client,
    base_url: String,
    #[allow(dead_code)] // May be needed for debugging/logging
    api_key: String,
    secret_key: String,
}

impl BinanceExecutionClient {
    pub fn new(api_key: String, secret_key: String, base_url: Option<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("X-MBX-APIKEY", HeaderValue::from_str(&api_key).unwrap());

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            base_url: base_url.unwrap_or_else(|| "https://api.binance.com".to_string()),
            api_key,
            secret_key,
        }
    }

    fn sign_query(&self, query: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(query.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    fn get_timestamp(&self) -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}

#[derive(Deserialize)]
#[allow(dead_code)] // Some fields are part of API contract but not used yet
struct BinanceOrderResponse {
    symbol: String,
    #[serde(rename = "orderId")]
    order_id: u64,
    #[serde(rename = "clientOrderId")]
    client_order_id: String,
    #[serde(rename = "transactTime")]
    transact_time: u64,
    price: String,
    #[serde(rename = "origQty")]
    orig_qty: String,
    #[serde(rename = "executedQty")]
    executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    cummulative_quote_qty: String,
    status: String,
    #[serde(rename = "timeInForce")]
    time_in_force: String,
    #[serde(rename = "type")]
    order_type: String,
    side: String,
}

#[async_trait]
impl ExecutionService for BinanceExecutionClient {
    async fn submit_order(&self, order: &Order) -> Result<String> {
        let endpoint = "/api/v3/order";
        let timestamp = self.get_timestamp();

        let side = match order.side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        let type_ = match order.order_type {
            OrderType::Market => "MARKET",
            OrderType::Limit => "LIMIT",
        };

        let mut query = format!(
            "symbol={}&side={}&type={}&quantity={}&timestamp={}",
            order.symbol, side, type_, order.quantity, timestamp
        );

        if order.order_type == OrderType::Limit {
            if let Some(price) = order.price {
                query.push_str(&format!("&price={}&timeInForce=GTC", price));
            } else {
                return Err(QuantError::DataValidation(
                    "Limit order must have price".to_string(),
                ));
            }
        }

        let signature = self.sign_query(&query);
        let url = format!(
            "{}{}{}?{}&signature={}",
            self.base_url, endpoint, "", query, signature
        );

        let response = self
            .client
            .post(&url)
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(QuantError::Api(format!("Binance API error: {}", text)));
        }

        let data: BinanceOrderResponse = response
            .json()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse response: {}", e)))?;

        Ok(data.order_id.to_string())
    }

    async fn cancel_order(&self, order_id: &str, symbol: &str) -> Result<()> {
        let endpoint = "/api/v3/order";
        let timestamp = self.get_timestamp();

        let query = format!(
            "symbol={}&orderId={}&timestamp={}",
            symbol, order_id, timestamp
        );

        let signature = self.sign_query(&query);
        let url = format!(
            "{}{}{}?{}&signature={}",
            self.base_url, endpoint, "", query, signature
        );

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(QuantError::Api(format!("Binance API error: {}", text)));
        }

        Ok(())
    }

    async fn get_order(&self, order_id: &str, symbol: &str) -> Result<Order> {
        let endpoint = "/api/v3/order";
        let timestamp = self.get_timestamp();

        let query = format!(
            "symbol={}&orderId={}&timestamp={}",
            symbol, order_id, timestamp
        );

        let signature = self.sign_query(&query);
        let url = format!(
            "{}{}{}?{}&signature={}",
            self.base_url, endpoint, "", query, signature
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| QuantError::Api(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(QuantError::Api(format!("Binance API error: {}", text)));
        }

        let data: BinanceOrderResponse = response
            .json()
            .await
            .map_err(|e| QuantError::Parse(format!("Failed to parse response: {}", e)))?;

        // Convert Binance response to core::Order
        let status = match data.status.as_str() {
            "NEW" => OrderStatus::New,
            "PARTIALLY_FILLED" => OrderStatus::PartiallyFilled,
            "FILLED" => OrderStatus::Filled,
            "CANCELED" => OrderStatus::Canceled,
            "REJECTED" => OrderStatus::Rejected,
            _ => OrderStatus::New, // Default/Fallback
        };

        let side = match data.side.as_str() {
            "BUY" => OrderSide::Buy,
            "SELL" => OrderSide::Sell,
            _ => OrderSide::Buy,
        };

        let order_type = match data.order_type.as_str() {
            "MARKET" => OrderType::Market,
            "LIMIT" => OrderType::Limit,
            _ => OrderType::Limit,
        };

        Ok(Order {
            id: data.order_id.to_string(),
            symbol: data.symbol,
            side,
            order_type,
            quantity: data.orig_qty.parse().unwrap_or(0.0),
            price: data.price.parse().ok(),
            status,
            timestamp: Utc::now(), // Note: Binance gives transact_time, but we use now() for simplicity or parse it
        })
    }
}
