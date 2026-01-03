//! Reports API Module
//!
//! Provides REST endpoints for trade journal, performance reports, and tax reporting.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use alphafield_backtest::{
    CostBasisMethod, JournalEntry, PerformanceReport, ReportPeriod, TaxCalculator, Trade,
};

use crate::api::AppState;

// ===========================
// Trade Journal API
// ===========================

/// Request to add a journal entry
#[derive(Debug, Deserialize)]
pub struct AddJournalEntryRequest {
    pub trade_id: String,
    pub notes: String,
    pub tags: Vec<String>,
}

/// Request to update a journal entry
#[derive(Debug, Deserialize)]
pub struct UpdateJournalEntryRequest {
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// List journal entries
pub async fn list_journal(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    // In a real app with persistence, we would load from DB
    // For now, we return empty or mocked data if we had it
    // This is a placeholder as the journal is currently ephemeral in memory
    // and would be lost on restart without DB persistence
    Json(Vec::<JournalEntry>::new())
}

/// Add a new journal entry
pub async fn add_journal_entry(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<AddJournalEntryRequest>,
) -> impl IntoResponse {
    // Placeholder - would integrate with persistent storage
    info!("Adding journal entry for trade {}", req.trade_id);
    StatusCode::CREATED
}

/// Update a journal entry
pub async fn update_journal_entry(
    Path(id): Path<String>,
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<UpdateJournalEntryRequest>,
) -> impl IntoResponse {
    info!("Updating journal entry {}", id);
    StatusCode::OK
}

/// Delete a journal entry
pub async fn delete_journal_entry(
    Path(id): Path<String>,
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("Deleting journal entry {}", id);
    StatusCode::OK
}

// ===========================
// Performance Reports API
// ===========================

/// Request to generate a performance report
#[derive(Debug, Deserialize)]
pub struct GenerateReportRequest {
    pub trades: Vec<Trade>,
    pub period: ReportPeriod,
    pub equity_history: Option<Vec<(i64, f64)>>,
}

/// Generate a performance summary
pub async fn generate_summary(Json(req): Json<GenerateReportRequest>) -> Json<PerformanceReport> {
    info!("Generating performance report for period {:?}", req.period);

    let report = if let Some(history) = req.equity_history {
        PerformanceReport::generate_with_equity(&req.trades, req.period, &history)
    } else {
        PerformanceReport::generate(&req.trades, req.period)
    };

    Json(report)
}

/// Request to export report as CSV
#[derive(Debug, Deserialize)]
pub struct ExportReportRequest {
    pub trades: Vec<Trade>,
    pub period: ReportPeriod,
}

/// Export report as CSV
pub async fn export_report_csv(Json(req): Json<ExportReportRequest>) -> impl IntoResponse {
    info!("Exporting performance report CSV");
    let report = PerformanceReport::generate(&req.trades, req.period);
    let csv = report.to_csv();

    ([(axum::http::header::CONTENT_TYPE, "text/csv")], csv)
}

/// Request for strategy breakdown
#[derive(Debug, Deserialize)]
pub struct StrategyBreakdownRequest {
    pub trades: Vec<Trade>,
}

/// Get strategy breakdown
pub async fn get_strategy_breakdown(
    Json(req): Json<StrategyBreakdownRequest>,
) -> Json<PerformanceReport> {
    info!("Generating strategy breakdown");
    // For now reusing generate logic with all-time period
    let report = PerformanceReport::generate(&req.trades, ReportPeriod::AllTime);
    Json(report)
}

// ===========================
// Tax Reporting API
// ===========================

/// Request to calculate tax
#[derive(Debug, Deserialize)]
pub struct CalculateTaxRequest {
    pub trades: Vec<Trade>,
    pub method: CostBasisMethod,
}

/// Calculate tax report
pub async fn calculate_tax(Json(req): Json<CalculateTaxRequest>) -> impl IntoResponse {
    info!("Calculating tax report using {:?}", req.method);
    let summary = TaxCalculator::calculate_from_trades(&req.trades, req.method);
    Json(summary)
}

/// Export tax report as CSV
pub async fn export_tax_csv(Json(req): Json<CalculateTaxRequest>) -> impl IntoResponse {
    info!("Exporting tax report CSV");
    let summary = TaxCalculator::calculate_from_trades(&req.trades, req.method);
    let csv = summary.to_csv();

    ([(axum::http::header::CONTENT_TYPE, "text/csv")], csv)
}
