//! WebSocket handler for real-time dashboard updates
//!
//! Provides broadcast hub for live portfolio, position, and trade updates.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Message types broadcast to WebSocket clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DashboardMessage {
    /// Portfolio summary update
    Portfolio(PortfolioUpdate),
    /// Position update (single position changed)
    Position(PositionUpdate),
    /// All positions snapshot
    Positions(Vec<PositionUpdate>),
    /// New trade executed
    Trade(TradeUpdate),
    /// System status/log message
    Log(LogMessage),
    /// Heartbeat/ping
    Heartbeat { timestamp: i64 },
    /// Trading engine status
    EngineStatus(EngineStatusUpdate),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioUpdate {
    pub total_value: f64,
    pub cash: f64,
    pub positions_value: f64,
    pub pnl: f64,
    pub pnl_percent: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdate {
    pub symbol: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub pnl: f64,
    pub pnl_percent: f64,
    /// Position side: "Long" or "Short"
    pub side: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeUpdate {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
    pub pnl: Option<f64>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub level: String,
    pub message: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatusUpdate {
    pub running: bool,
    pub mode: String, // "paper" or "live"
    pub strategy: Option<String>,
    pub uptime_secs: u64,
}

/// Broadcast hub for dashboard updates
#[derive(Debug, Clone)]
pub struct DashboardHub {
    sender: broadcast::Sender<DashboardMessage>,
}

impl DashboardHub {
    /// Create a new hub with specified capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Get a receiver for this hub
    pub fn subscribe(&self) -> broadcast::Receiver<DashboardMessage> {
        self.sender.subscribe()
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, msg: DashboardMessage) {
        // Ignore send errors (no receivers is fine)
        let _ = self.sender.send(msg);
    }

    /// Broadcast a portfolio update
    pub fn update_portfolio(&self, update: PortfolioUpdate) {
        self.broadcast(DashboardMessage::Portfolio(update));
    }

    /// Broadcast a position update
    pub fn update_position(&self, update: PositionUpdate) {
        self.broadcast(DashboardMessage::Position(update));
    }

    /// Broadcast all positions
    pub fn update_all_positions(&self, positions: Vec<PositionUpdate>) {
        self.broadcast(DashboardMessage::Positions(positions));
    }

    /// Broadcast a new trade
    pub fn new_trade(&self, trade: TradeUpdate) {
        self.broadcast(DashboardMessage::Trade(trade));
    }

    /// Broadcast a log message
    pub fn log(&self, level: &str, message: &str) {
        self.broadcast(DashboardMessage::Log(LogMessage {
            level: level.to_string(),
            message: message.to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        }));
    }

    /// Broadcast engine status
    pub fn update_engine_status(&self, status: EngineStatusUpdate) {
        self.broadcast(DashboardMessage::EngineStatus(status));
    }

    /// Send heartbeat
    pub fn heartbeat(&self) {
        self.broadcast(DashboardMessage::Heartbeat {
            timestamp: chrono::Utc::now().timestamp_millis(),
        });
    }
}

impl Default for DashboardHub {
    fn default() -> Self {
        Self::new(256)
    }
}

/// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(hub): State<Arc<DashboardHub>>,
) -> impl IntoResponse {
    info!("New WebSocket connection request");
    ws.on_upgrade(move |socket| handle_socket(socket, hub))
}

/// Handle an individual WebSocket connection
async fn handle_socket(socket: WebSocket, hub: Arc<DashboardHub>) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to broadcast updates
    let mut rx = hub.subscribe();

    // Log connection
    hub.log("info", "Client connected to WebSocket");

    // Spawn task to forward broadcast messages to this client
    let hub_clone = hub.clone();
    let send_task = tokio::spawn(async move {
        loop {
            match rx.recv().await {
                Ok(msg) => {
                    let json = match serde_json::to_string(&msg) {
                        Ok(j) => j,
                        Err(e) => {
                            error!("Failed to serialize message: {}", e);
                            continue;
                        }
                    };

                    if sender.send(Message::Text(json)).await.is_err() {
                        debug!("Client disconnected (send failed)");
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Client lagged, skipped {} messages", n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    debug!("Broadcast channel closed");
                    break;
                }
            }
        }
    });

    // Handle incoming messages from client
    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    debug!("Received from client: {}", text);
                    // Handle client commands here (e.g., start/stop trading)
                    if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                        handle_client_command(cmd, &hub_clone).await;
                    }
                }
                Ok(Message::Ping(data)) => {
                    debug!("Received ping");
                    // Pong is handled automatically by axum
                    let _ = data;
                }
                Ok(Message::Pong(_)) => {
                    debug!("Received pong");
                }
                Ok(Message::Close(_)) => {
                    info!("Client requested close");
                    break;
                }
                Ok(Message::Binary(_)) => {
                    // Ignore binary messages
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => debug!("Send task completed"),
        _ = recv_task => debug!("Receive task completed"),
    }

    hub.log("info", "Client disconnected from WebSocket");
}

/// Commands that can be sent from the client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum ClientCommand {
    /// Start the trading engine
    Start { strategy: String, mode: String },
    /// Stop the trading engine
    Stop,
    /// Panic close all positions
    PanicClose,
    /// Request current state snapshot
    Snapshot,
    /// Ping for connection health
    Ping,
}

/// Handle commands from WebSocket clients
async fn handle_client_command(cmd: ClientCommand, hub: &DashboardHub) {
    match cmd {
        ClientCommand::Start { strategy, mode } => {
            info!(
                "Client requested start: strategy={}, mode={}",
                strategy, mode
            );
            hub.log("info", &format!("Starting {} in {} mode", strategy, mode));
            // TODO: Actually start the trading engine
            hub.update_engine_status(EngineStatusUpdate {
                running: true,
                mode,
                strategy: Some(strategy),
                uptime_secs: 0,
            });
        }
        ClientCommand::Stop => {
            info!("Client requested stop");
            hub.log("info", "Stopping trading engine");
            // TODO: Actually stop the trading engine
            hub.update_engine_status(EngineStatusUpdate {
                running: false,
                mode: "paper".to_string(),
                strategy: None,
                uptime_secs: 0,
            });
        }
        ClientCommand::PanicClose => {
            warn!("Client requested PANIC CLOSE");
            hub.log("warn", "PANIC CLOSE requested - closing all positions");
            // TODO: Actually close all positions
        }
        ClientCommand::Snapshot => {
            debug!("Client requested snapshot");
            // TODO: Send current state
        }
        ClientCommand::Ping => {
            hub.heartbeat();
        }
    }
}

/// Start heartbeat background task
pub fn start_heartbeat_task(hub: Arc<DashboardHub>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            hub.heartbeat();
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = DashboardMessage::Portfolio(PortfolioUpdate {
            total_value: 100000.0,
            cash: 50000.0,
            positions_value: 50000.0,
            pnl: 5000.0,
            pnl_percent: 5.0,
            timestamp: 1234567890,
        });

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Portfolio"));
        assert!(json.contains("100000"));
    }

    #[test]
    fn test_hub_broadcast() {
        let hub = DashboardHub::new(16);
        let mut rx = hub.subscribe();

        hub.heartbeat();

        // Should receive the heartbeat
        let result = rx.try_recv();
        assert!(result.is_ok());
    }
}
