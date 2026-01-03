//! Trade Journal Module
//!
//! Provides data structures for maintaining a trade journal with notes,
//! tags, and metadata for each trade. Supports persistence via JSON serialization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::trade::Trade;

/// Represents a journal entry associated with a trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Unique identifier for this journal entry
    pub id: String,
    /// Reference to the trade (by entry timestamp + symbol as composite key)
    pub trade_id: String,
    /// Symbol traded
    pub symbol: String,
    /// Entry timestamp of the original trade
    pub trade_entry_time: DateTime<Utc>,
    /// Exit timestamp of the original trade
    pub trade_exit_time: DateTime<Utc>,
    /// Trade P&L for quick reference
    pub trade_pnl: f64,
    /// User notes about this trade
    pub notes: String,
    /// Tags for categorization (e.g., "momentum", "reversal", "breakout")
    pub tags: Vec<String>,
    /// Path to screenshot if captured (optional, future feature)
    pub screenshot_path: Option<String>,
    /// When this journal entry was created
    pub created_at: DateTime<Utc>,
    /// When this journal entry was last updated
    pub updated_at: DateTime<Utc>,
}

impl JournalEntry {
    /// Create a new journal entry from a trade
    pub fn from_trade(trade: &Trade) -> Self {
        let trade_id = format!("{}_{}", trade.symbol, trade.entry_time.timestamp());
        Self {
            id: Uuid::new_v4().to_string(),
            trade_id,
            symbol: trade.symbol.clone(),
            trade_entry_time: trade.entry_time,
            trade_exit_time: trade.exit_time,
            trade_pnl: trade.pnl,
            notes: String::new(),
            tags: Vec::new(),
            screenshot_path: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Update notes for this entry
    pub fn set_notes(&mut self, notes: String) {
        self.notes = notes;
        self.updated_at = Utc::now();
    }

    /// Add a tag to this entry
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a tag from this entry
    pub fn remove_tag(&mut self, tag: &str) {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// Set tags, replacing existing ones
    pub fn set_tags(&mut self, tags: Vec<String>) {
        self.tags = tags;
        self.updated_at = Utc::now();
    }
}

/// Trade journal for managing journal entries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TradeJournal {
    /// All journal entries indexed by their ID
    entries: HashMap<String, JournalEntry>,
    /// Index from trade_id to entry_id for quick lookup
    #[serde(default)]
    trade_index: HashMap<String, String>,
}

impl TradeJournal {
    /// Create a new empty journal
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            trade_index: HashMap::new(),
        }
    }

    /// Add a new journal entry
    pub fn add_entry(&mut self, entry: JournalEntry) -> String {
        let id = entry.id.clone();
        let trade_id = entry.trade_id.clone();
        self.trade_index.insert(trade_id, id.clone());
        self.entries.insert(id.clone(), entry);
        id
    }

    /// Create and add a journal entry from a trade
    pub fn add_from_trade(&mut self, trade: &Trade) -> String {
        let entry = JournalEntry::from_trade(trade);
        self.add_entry(entry)
    }

    /// Get an entry by ID
    pub fn get_entry(&self, id: &str) -> Option<&JournalEntry> {
        self.entries.get(id)
    }

    /// Get a mutable entry by ID
    pub fn get_entry_mut(&mut self, id: &str) -> Option<&mut JournalEntry> {
        self.entries.get_mut(id)
    }

    /// Get entry by trade ID
    pub fn get_by_trade_id(&self, trade_id: &str) -> Option<&JournalEntry> {
        self.trade_index
            .get(trade_id)
            .and_then(|id| self.entries.get(id))
    }

    /// Update an entry's notes
    pub fn update_notes(&mut self, id: &str, notes: String) -> bool {
        if let Some(entry) = self.entries.get_mut(id) {
            entry.set_notes(notes);
            true
        } else {
            false
        }
    }

    /// Update an entry's tags
    pub fn update_tags(&mut self, id: &str, tags: Vec<String>) -> bool {
        if let Some(entry) = self.entries.get_mut(id) {
            entry.set_tags(tags);
            true
        } else {
            false
        }
    }

    /// Delete an entry by ID
    pub fn delete_entry(&mut self, id: &str) -> Option<JournalEntry> {
        if let Some(entry) = self.entries.remove(id) {
            self.trade_index.remove(&entry.trade_id);
            Some(entry)
        } else {
            None
        }
    }

    /// List all entries, optionally filtered by symbol
    pub fn list_entries(&self, symbol: Option<&str>) -> Vec<&JournalEntry> {
        let mut entries: Vec<_> = self
            .entries
            .values()
            .filter(|e| symbol.is_none() || symbol == Some(e.symbol.as_str()))
            .collect();
        // Sort by trade entry time, most recent first
        entries.sort_by(|a, b| b.trade_entry_time.cmp(&a.trade_entry_time));
        entries
    }

    /// List entries filtered by tag
    pub fn list_by_tag(&self, tag: &str) -> Vec<&JournalEntry> {
        let mut entries: Vec<_> = self
            .entries
            .values()
            .filter(|e| e.tags.contains(&tag.to_string()))
            .collect();
        entries.sort_by(|a, b| b.trade_entry_time.cmp(&a.trade_entry_time));
        entries
    }

    /// Get count of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if journal is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get all unique tags across all entries
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<_> = self
            .entries
            .values()
            .flat_map(|e| e.tags.iter().cloned())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trade::TradeSide;
    use chrono::TimeZone;

    fn make_test_trade(symbol: &str, pnl: f64, entry_secs: i64) -> Trade {
        let entry = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
            + chrono::Duration::seconds(entry_secs);
        let exit = entry + chrono::Duration::hours(1);
        Trade {
            symbol: symbol.to_string(),
            side: TradeSide::Long,
            entry_time: entry,
            exit_time: exit,
            entry_price: 100.0,
            exit_price: if pnl > 0.0 { 110.0 } else { 90.0 },
            quantity: 1.0,
            pnl,
            fees: 0.1,
            mae: 5.0,
            mfe: 15.0,
            duration_secs: 3600,
            exit_reason: Some("Signal".to_string()),
        }
    }

    #[test]
    fn test_journal_entry_from_trade() {
        let trade = make_test_trade("BTC", 100.0, 0);
        let entry = JournalEntry::from_trade(&trade);

        assert_eq!(entry.symbol, "BTC");
        assert_eq!(entry.trade_pnl, 100.0);
        assert!(entry.notes.is_empty());
        assert!(entry.tags.is_empty());
    }

    #[test]
    fn test_journal_crud() {
        let mut journal = TradeJournal::new();
        assert!(journal.is_empty());

        // Add entry
        let trade = make_test_trade("ETH", 50.0, 100);
        let id = journal.add_from_trade(&trade);
        assert_eq!(journal.len(), 1);

        // Get entry
        let entry = journal.get_entry(&id).unwrap();
        assert_eq!(entry.symbol, "ETH");

        // Update notes
        journal.update_notes(&id, "Good breakout trade".to_string());
        let entry = journal.get_entry(&id).unwrap();
        assert_eq!(entry.notes, "Good breakout trade");

        // Update tags
        journal.update_tags(&id, vec!["breakout".to_string(), "trend".to_string()]);
        let entry = journal.get_entry(&id).unwrap();
        assert_eq!(entry.tags.len(), 2);

        // Delete entry
        let deleted = journal.delete_entry(&id);
        assert!(deleted.is_some());
        assert!(journal.is_empty());
    }

    #[test]
    fn test_journal_list_and_filter() {
        let mut journal = TradeJournal::new();

        let trade1 = make_test_trade("BTC", 100.0, 0);
        let trade2 = make_test_trade("ETH", 50.0, 100);
        let trade3 = make_test_trade("BTC", -30.0, 200);

        let id1 = journal.add_from_trade(&trade1);
        let _id2 = journal.add_from_trade(&trade2);
        let id3 = journal.add_from_trade(&trade3);

        // Add tags
        journal.update_tags(&id1, vec!["winner".to_string()]);
        journal.update_tags(&id3, vec!["loser".to_string()]);

        // List all
        assert_eq!(journal.list_entries(None).len(), 3);

        // Filter by symbol
        assert_eq!(journal.list_entries(Some("BTC")).len(), 2);
        assert_eq!(journal.list_entries(Some("ETH")).len(), 1);

        // Filter by tag
        assert_eq!(journal.list_by_tag("winner").len(), 1);
    }

    #[test]
    fn test_journal_serialization() {
        let mut journal = TradeJournal::new();
        let trade = make_test_trade("SOL", 75.0, 0);
        let id = journal.add_from_trade(&trade);
        journal.update_notes(&id, "Test note".to_string());

        // Serialize
        let json = journal.to_json().unwrap();
        assert!(json.contains("SOL"));
        assert!(json.contains("Test note"));

        // Deserialize
        let restored = TradeJournal::from_json(&json).unwrap();
        assert_eq!(restored.len(), 1);
        let entry = restored.get_entry(&id).unwrap();
        assert_eq!(entry.notes, "Test note");
    }

    #[test]
    fn test_get_by_trade_id() {
        let mut journal = TradeJournal::new();
        let trade = make_test_trade("AVAX", 25.0, 500);
        let trade_id = format!("{}_{}", trade.symbol, trade.entry_time.timestamp());

        journal.add_from_trade(&trade);

        let entry = journal.get_by_trade_id(&trade_id);
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().symbol, "AVAX");
    }
}
