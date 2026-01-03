//! Tax Reporting Module
//!
//! Provides cost basis tracking using FIFO and LIFO methods,
//! realized gains/losses calculation by tax year, and CSV export
//! formatted for tax software compatibility.

use chrono::{DateTime, Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::trade::{Trade, TradeSide};

/// Cost basis accounting method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CostBasisMethod {
    /// First-In, First-Out
    #[default]
    FIFO,
    /// Last-In, First-Out
    LIFO,
}

/// Represents a tax lot (a purchase of an asset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLot {
    /// Symbol
    pub symbol: String,
    /// Purchase date
    pub acquired_date: DateTime<Utc>,
    /// Original quantity purchased
    pub original_quantity: f64,
    /// Remaining quantity (after partial sales)
    pub remaining_quantity: f64,
    /// Cost per unit at purchase
    pub cost_per_unit: f64,
    /// Total cost basis for this lot
    pub cost_basis: f64,
    /// Fees paid on acquisition
    pub acquisition_fees: f64,
}

impl TaxLot {
    /// Create a new tax lot from a long trade entry
    pub fn from_trade(trade: &Trade) -> Self {
        Self {
            symbol: trade.symbol.clone(),
            acquired_date: trade.entry_time,
            original_quantity: trade.quantity,
            remaining_quantity: trade.quantity,
            cost_per_unit: trade.entry_price,
            cost_basis: trade.entry_price * trade.quantity,
            acquisition_fees: trade.fees / 2.0, // Assume half of fees are on entry
        }
    }

    /// Consume quantity from this lot, returns (quantity_consumed, cost_basis_consumed)
    pub fn consume(&mut self, quantity: f64) -> (f64, f64) {
        let consumed = quantity.min(self.remaining_quantity);
        let cost_consumed = consumed * self.cost_per_unit;
        self.remaining_quantity -= consumed;
        (consumed, cost_consumed)
    }

    /// Check if this lot is exhausted
    pub fn is_empty(&self) -> bool {
        self.remaining_quantity <= 0.0
    }
}

/// Represents a taxable event (sale of an asset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxableEvent {
    /// Symbol sold
    pub symbol: String,
    /// Date of sale
    pub sale_date: DateTime<Utc>,
    /// Date(s) of acquisition (may span multiple lots)
    pub acquired_dates: Vec<DateTime<Utc>>,
    /// Quantity sold
    pub quantity: f64,
    /// Proceeds from sale
    pub proceeds: f64,
    /// Cost basis of sold units
    pub cost_basis: f64,
    /// Realized gain or loss
    pub gain_loss: f64,
    /// Whether this is a short-term gain (held < 1 year)
    pub is_short_term: bool,
    /// Tax year
    pub tax_year: i32,
    /// Fees paid on sale
    pub sale_fees: f64,
}

impl TaxableEvent {
    /// Calculate if gain is short-term based on acquisition dates
    pub fn calculate_term(acquired_date: DateTime<Utc>, sale_date: DateTime<Utc>) -> bool {
        let duration = sale_date.signed_duration_since(acquired_date);
        duration.num_days() < 365
    }
}

/// Tax summary for a year
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaxYearSummary {
    /// Tax year
    pub year: i32,
    /// Total short-term gains
    pub short_term_gains: f64,
    /// Total short-term losses
    pub short_term_losses: f64,
    /// Net short-term gain/loss
    pub net_short_term: f64,
    /// Total long-term gains
    pub long_term_gains: f64,
    /// Total long-term losses
    pub long_term_losses: f64,
    /// Net long-term gain/loss
    pub net_long_term: f64,
    /// Total realized gain/loss
    pub total_net: f64,
    /// Number of taxable events
    pub event_count: usize,
    /// Total proceeds
    pub total_proceeds: f64,
    /// Total cost basis
    pub total_cost_basis: f64,
}

impl TaxYearSummary {
    /// Calculate summary from taxable events
    pub fn calculate(year: i32, events: &[&TaxableEvent]) -> Self {
        let mut summary = Self {
            year,
            event_count: events.len(),
            ..Default::default()
        };

        for event in events {
            summary.total_proceeds += event.proceeds;
            summary.total_cost_basis += event.cost_basis;

            if event.is_short_term {
                if event.gain_loss >= 0.0 {
                    summary.short_term_gains += event.gain_loss;
                } else {
                    summary.short_term_losses += event.gain_loss.abs();
                }
            } else if event.gain_loss >= 0.0 {
                summary.long_term_gains += event.gain_loss;
            } else {
                summary.long_term_losses += event.gain_loss.abs();
            }
        }

        summary.net_short_term = summary.short_term_gains - summary.short_term_losses;
        summary.net_long_term = summary.long_term_gains - summary.long_term_losses;
        summary.total_net = summary.net_short_term + summary.net_long_term;

        summary
    }
}

/// Complete tax report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxSummary {
    /// Cost basis method used
    pub method: CostBasisMethod,
    /// All taxable events
    pub events: Vec<TaxableEvent>,
    /// Summary by year
    pub yearly_summaries: Vec<TaxYearSummary>,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
}

impl TaxSummary {
    /// Export to CSV format compatible with tax software
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();

        // Header - Form 8949 compatible format
        csv.push_str("Description,Date Acquired,Date Sold,Proceeds,Cost Basis,Gain or Loss,Term\n");

        for event in &self.events {
            let acquired_str = if event.acquired_dates.len() == 1 {
                event.acquired_dates[0].format("%m/%d/%Y").to_string()
            } else {
                "Various".to_string()
            };

            let term = if event.is_short_term { "Short" } else { "Long" };

            csv.push_str(&format!(
                "{} {:.4},{},{},{:.2},{:.2},{:.2},{}\n",
                event.quantity,
                event.symbol,
                acquired_str,
                event.sale_date.format("%m/%d/%Y"),
                event.proceeds,
                event.cost_basis,
                event.gain_loss,
                term
            ));
        }

        // Add summary section
        csv.push_str("\nANNUAL SUMMARY\n");
        csv.push_str("Year,Short-Term Gains,Short-Term Losses,Net Short-Term,Long-Term Gains,Long-Term Losses,Net Long-Term,Total Net\n");

        for yearly in &self.yearly_summaries {
            csv.push_str(&format!(
                "{},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2},{:.2}\n",
                yearly.year,
                yearly.short_term_gains,
                yearly.short_term_losses,
                yearly.net_short_term,
                yearly.long_term_gains,
                yearly.long_term_losses,
                yearly.net_long_term,
                yearly.total_net
            ));
        }

        csv
    }
}

/// Tax calculator for computing realized gains/losses
#[derive(Debug, Clone)]
pub struct TaxCalculator {
    /// Cost basis method
    method: CostBasisMethod,
    /// Open tax lots by symbol
    lots: HashMap<String, Vec<TaxLot>>,
    /// Recorded taxable events
    events: Vec<TaxableEvent>,
}

impl TaxCalculator {
    /// Create a new tax calculator
    pub fn new(method: CostBasisMethod) -> Self {
        Self {
            method,
            lots: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Process a trade and update lots/events accordingly
    pub fn process_trade(&mut self, trade: &Trade) {
        match trade.side {
            TradeSide::Long => {
                // Buying - add a new lot
                let lot = TaxLot::from_trade(trade);
                self.lots.entry(trade.symbol.clone()).or_default().push(lot);
            }
            TradeSide::Short => {
                // Selling - consume from lots and record event
                // For now, treat short trades as sales of existing long positions
                self.process_sale(trade);
            }
        }
    }

    /// Process a completed trade (both entry and exit)
    pub fn process_completed_trade(&mut self, trade: &Trade) {
        // For a completed trade, entry is the buy, exit is the sell
        // Add lot for the entry
        let lot = TaxLot::from_trade(trade);
        self.lots.entry(trade.symbol.clone()).or_default().push(lot);

        // Process the exit as a sale
        let sale_proceeds = trade.exit_price * trade.quantity;
        let sale_fees = trade.fees / 2.0; // Half of fees on exit

        self.record_sale(
            &trade.symbol,
            trade.exit_time,
            trade.quantity,
            sale_proceeds,
            sale_fees,
        );
    }

    fn process_sale(&mut self, trade: &Trade) {
        let proceeds = trade.exit_price * trade.quantity;
        let fees = trade.fees / 2.0;

        self.record_sale(
            &trade.symbol,
            trade.exit_time,
            trade.quantity,
            proceeds,
            fees,
        );
    }

    fn record_sale(
        &mut self,
        symbol: &str,
        sale_date: DateTime<Utc>,
        quantity: f64,
        proceeds: f64,
        fees: f64,
    ) {
        let lots = match self.lots.get_mut(symbol) {
            Some(l) => l,
            None => return, // No lots to sell from
        };

        let mut remaining = quantity;
        let mut total_cost_basis = 0.0;
        let mut acquired_dates = Vec::new();
        let mut earliest_date = sale_date;

        // Select lots based on method
        while remaining > 0.0 && !lots.is_empty() {
            let lot_idx = match self.method {
                CostBasisMethod::FIFO => 0,              // First lot
                CostBasisMethod::LIFO => lots.len() - 1, // Last lot
            };

            let lot = &mut lots[lot_idx];
            let (consumed, cost) = lot.consume(remaining);
            remaining -= consumed;
            total_cost_basis += cost;
            acquired_dates.push(lot.acquired_date);

            if lot.acquired_date < earliest_date {
                earliest_date = lot.acquired_date;
            }

            if lot.is_empty() {
                lots.remove(lot_idx);
            }
        }

        if !acquired_dates.is_empty() {
            let is_short_term = TaxableEvent::calculate_term(earliest_date, sale_date);
            let gain_loss = proceeds - total_cost_basis - fees;

            self.events.push(TaxableEvent {
                symbol: symbol.to_string(),
                sale_date,
                acquired_dates,
                quantity: quantity - remaining,
                proceeds,
                cost_basis: total_cost_basis,
                gain_loss,
                is_short_term,
                tax_year: sale_date.year(),
                sale_fees: fees,
            });
        }
    }

    /// Calculate tax from a list of completed trades
    pub fn calculate_from_trades(trades: &[Trade], method: CostBasisMethod) -> TaxSummary {
        let mut calc = Self::new(method);

        for trade in trades {
            calc.process_completed_trade(trade);
        }

        calc.generate_summary()
    }

    /// Generate the final tax summary
    pub fn generate_summary(&self) -> TaxSummary {
        // Group events by year
        let mut by_year: HashMap<i32, Vec<&TaxableEvent>> = HashMap::new();
        for event in &self.events {
            by_year.entry(event.tax_year).or_default().push(event);
        }

        // Calculate yearly summaries
        let mut yearly_summaries: Vec<TaxYearSummary> = by_year
            .into_iter()
            .map(|(year, events)| TaxYearSummary::calculate(year, &events))
            .collect();

        yearly_summaries.sort_by_key(|s| s.year);

        TaxSummary {
            method: self.method,
            events: self.events.clone(),
            yearly_summaries,
            generated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_trade(
        symbol: &str,
        side: TradeSide,
        entry_time: DateTime<Utc>,
        exit_time: DateTime<Utc>,
        entry_price: f64,
        exit_price: f64,
        quantity: f64,
    ) -> Trade {
        let pnl = match side {
            TradeSide::Long => (exit_price - entry_price) * quantity,
            TradeSide::Short => (entry_price - exit_price) * quantity,
        };
        Trade {
            symbol: symbol.to_string(),
            side,
            entry_time,
            exit_time,
            entry_price,
            exit_price,
            quantity,
            pnl,
            fees: 1.0,
            mae: 5.0,
            mfe: 15.0,
            duration_secs: 3600,
            exit_reason: Some("Signal".to_string()),
        }
    }

    #[test]
    fn test_fifo_cost_basis() {
        // Buy 1 BTC at $100, then 1 BTC at $150, then sell 1 BTC at $200
        let trades = vec![
            make_trade(
                "BTC",
                TradeSide::Long,
                Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap(),
                100.0,
                200.0,
                1.0,
            ),
            make_trade(
                "BTC",
                TradeSide::Long,
                Utc.with_ymd_and_hms(2024, 2, 1, 10, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 2, 1, 11, 0, 0).unwrap(),
                150.0,
                200.0,
                1.0,
            ),
        ];

        let summary = TaxCalculator::calculate_from_trades(&trades, CostBasisMethod::FIFO);

        assert_eq!(summary.events.len(), 2);

        // First sale: bought at $100, sold at $200 = $100 gain (minus fees)
        let event1 = &summary.events[0];
        assert!((event1.proceeds - 200.0).abs() < 0.01);
        assert!((event1.cost_basis - 100.0).abs() < 0.01);

        // Second sale: bought at $150, sold at $200 = $50 gain (minus fees)
        let event2 = &summary.events[1];
        assert!((event2.proceeds - 200.0).abs() < 0.01);
        assert!((event2.cost_basis - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_lifo_cost_basis() {
        // Same scenario but with LIFO
        let trades = vec![
            make_trade(
                "BTC",
                TradeSide::Long,
                Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 3, 1, 11, 0, 0).unwrap(), // Sold later
                100.0,
                200.0,
                1.0,
            ),
            make_trade(
                "BTC",
                TradeSide::Long,
                Utc.with_ymd_and_hms(2024, 2, 1, 10, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 2, 15, 11, 0, 0).unwrap(), // Sold first with LIFO
                150.0,
                200.0,
                1.0,
            ),
        ];

        let summary = TaxCalculator::calculate_from_trades(&trades, CostBasisMethod::LIFO);

        assert_eq!(summary.events.len(), 2);
    }

    #[test]
    fn test_short_vs_long_term() {
        // Short-term: held < 1 year
        let short_term_trade = make_trade(
            "ETH",
            TradeSide::Long,
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap(), // 5 months
            100.0,
            150.0,
            1.0,
        );

        // Long-term: held >= 1 year
        let long_term_trade = make_trade(
            "ETH",
            TradeSide::Long,
            Utc.with_ymd_and_hms(2023, 1, 1, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap(), // 17 months
            100.0,
            150.0,
            1.0,
        );

        let summary = TaxCalculator::calculate_from_trades(
            &[short_term_trade, long_term_trade],
            CostBasisMethod::FIFO,
        );

        let short_term_events: Vec<_> = summary.events.iter().filter(|e| e.is_short_term).collect();
        let long_term_events: Vec<_> = summary.events.iter().filter(|e| !e.is_short_term).collect();

        assert_eq!(short_term_events.len(), 1);
        assert_eq!(long_term_events.len(), 1);
    }

    #[test]
    fn test_yearly_summary() {
        let trades = vec![
            make_trade(
                "BTC",
                TradeSide::Long,
                Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap(),
                100.0,
                150.0,
                1.0,
            ),
            make_trade(
                "ETH",
                TradeSide::Long,
                Utc.with_ymd_and_hms(2024, 7, 1, 10, 0, 0).unwrap(),
                Utc.with_ymd_and_hms(2024, 12, 1, 10, 0, 0).unwrap(),
                200.0,
                180.0, // Loss
                1.0,
            ),
        ];

        let summary = TaxCalculator::calculate_from_trades(&trades, CostBasisMethod::FIFO);

        assert_eq!(summary.yearly_summaries.len(), 1);
        let year_2024 = &summary.yearly_summaries[0];
        assert_eq!(year_2024.year, 2024);
        assert_eq!(year_2024.event_count, 2);
    }

    #[test]
    fn test_csv_export() {
        let trades = vec![make_trade(
            "BTC",
            TradeSide::Long,
            Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 6, 1, 10, 0, 0).unwrap(),
            100.0,
            150.0,
            1.0,
        )];

        let summary = TaxCalculator::calculate_from_trades(&trades, CostBasisMethod::FIFO);
        let csv = summary.to_csv();

        assert!(csv.contains("Description,Date Acquired"));
        assert!(csv.contains("BTC"));
        assert!(csv.contains("ANNUAL SUMMARY"));
        assert!(csv.contains("2024"));
    }
}
