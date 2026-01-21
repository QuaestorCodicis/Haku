use axum::{
    extract::State,
    response::{Html, IntoResponse, Response, sse::{Event, Sse}},
    routing::get,
    Json, Router,
};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_stream::{Stream, StreamExt as _};
use tokio::time::{interval, Duration};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tracing::{info, error};

use crate::portfolio_monitor::DailyStats;
use crate::persistence::{TradeHistory, SerializableClosedTrade};

#[derive(Clone)]
pub struct DashboardState {
    pub stats: Arc<RwLock<DailyStats>>,
    pub trade_history: Arc<RwLock<TradeHistory>>,
}

pub struct DashboardServer {
    port: u16,
    state: DashboardState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

impl DashboardServer {
    pub fn new(port: u16, trade_history: TradeHistory) -> Self {
        let state = DashboardState {
            stats: Arc::new(RwLock::new(DailyStats::default())),
            trade_history: Arc::new(RwLock::new(trade_history)),
        };

        Self { port, state }
    }

    pub fn get_state(&self) -> DashboardState {
        self.state.clone()
    }

    pub async fn start(self) {
        info!("🌐 Starting web dashboard on http://localhost:{}", self.port);

        let app = Router::new()
            .route("/", get(serve_dashboard))
            .route("/api/stats", get(get_stats))
            .route("/api/trades", get(get_trades))
            .route("/api/stream", get(sse_handler))
            .layer(CorsLayer::permissive())
            .with_state(self.state);

        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port))
            .await
            .expect("Failed to bind dashboard server");

        info!("📊 Dashboard ready at http://localhost:{}", self.port);

        axum::serve(listener, app)
            .await
            .expect("Failed to start dashboard server");
    }
}

async fn serve_dashboard() -> Html<&'static str> {
    Html(include_str!("../static/dashboard.html"))
}

async fn get_stats(
    State(state): State<DashboardState>,
) -> Json<ApiResponse<DailyStats>> {
    let stats = state.stats.read().await.clone();
    Json(ApiResponse {
        success: true,
        data: stats,
    })
}

async fn get_trades(
    State(state): State<DashboardState>,
) -> Json<ApiResponse<Vec<SerializableClosedTrade>>> {
    let history = state.trade_history.read().await;
    let trades = history.closed_trades.clone();
    Json(ApiResponse {
        success: true,
        data: trades,
    })
}

async fn sse_handler(
    State(state): State<DashboardState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut tick = interval(Duration::from_secs(5));

        loop {
            tick.tick().await;

            let stats = state.stats.read().await.clone();

            match serde_json::to_string(&stats) {
                Ok(json) => {
                    yield Ok(Event::default().data(json));
                }
                Err(e) => {
                    error!("Failed to serialize stats: {}", e);
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

// Update stats helper
impl DashboardState {
    pub async fn update_stats(&self, stats: &DailyStats) {
        *self.stats.write().await = stats.clone();
    }

    pub async fn update_trade_history(&self, history: &TradeHistory) {
        *self.trade_history.write().await = history.clone();
    }
}
