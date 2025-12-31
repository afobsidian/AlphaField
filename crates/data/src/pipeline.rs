use crate::database::DatabaseClient;
use alphafield_core::{Bar, QuantError, Result, Tick};
use tokio::sync::broadcast;
use tracing::{error, info};

/// Events that flow through the data pipeline
#[derive(Clone, Debug)]
pub enum MarketEvent {
    /// A completed OHLCV bar
    Bar(Bar),
    /// A real-time trade tick
    Tick(Tick),
}

/// The central hub for distributing market data
pub struct DataPipeline {
    sender: broadcast::Sender<MarketEvent>,
}

impl DataPipeline {
    /// Creates a new data pipeline with a specified buffer size
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Subscribes to the pipeline to receive events
    pub fn subscribe(&self) -> broadcast::Receiver<MarketEvent> {
        self.sender.subscribe()
    }

    /// Publishes an event to all subscribers
    pub fn publish(&self, event: MarketEvent) -> Result<usize> {
        self.sender.send(event).map_err(|e| {
            QuantError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                format!("Failed to publish event: {}", e),
            ))
        })
    }
}

/// Service that persists market data to the database
pub struct DataPersister {
    client: DatabaseClient,
    receiver: broadcast::Receiver<MarketEvent>,
    symbol: String,
    timeframe: String,
}

impl DataPersister {
    /// Creates a new persister service
    pub fn new(
        client: DatabaseClient,
        pipeline: &DataPipeline,
        symbol: impl Into<String>,
        timeframe: impl Into<String>,
    ) -> Self {
        Self {
            client,
            receiver: pipeline.subscribe(),
            symbol: symbol.into(),
            timeframe: timeframe.into(),
        }
    }

    /// Runs the persistence loop
    pub async fn run(mut self) {
        info!(
            "Starting DataPersister for {} {}",
            self.symbol, self.timeframe
        );

        loop {
            match self.receiver.recv().await {
                Ok(event) => match event {
                    MarketEvent::Bar(bar) => {
                        // Persist bar
                        if let Err(e) = self
                            .client
                            .save_bars(&self.symbol, &self.timeframe, &[bar])
                            .await
                        {
                            error!("Failed to persist bar: {}", e);
                        } else {
                            info!("Persisted bar: {}", bar);
                        }
                    }
                    MarketEvent::Tick(_tick) => {
                        // Persist tick to trades table
                        if let Err(e) = self.client.save_tick(&self.symbol, &_tick).await {
                            error!("Failed to persist tick: {}", e);
                        } else {
                            info!("Persisted tick: {}", _tick);
                        }
                    }
                },
                Err(broadcast::error::RecvError::Lagged(count)) => {
                    error!("DataPersister lagged by {} events", count);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    info!("DataPipeline closed, stopping persister");
                    break;
                }
            }
        }
    }
}
