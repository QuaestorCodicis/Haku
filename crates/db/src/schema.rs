// Database schema definitions

pub const CREATE_TRADES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS trades (
    id TEXT PRIMARY KEY,
    wallet TEXT NOT NULL,
    token_mint TEXT NOT NULL,
    side TEXT NOT NULL,
    amount_in TEXT NOT NULL,
    amount_out TEXT NOT NULL,
    price_usd TEXT NOT NULL,
    market_cap_at_trade TEXT NOT NULL,
    signature TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    block_time INTEGER NOT NULL,
    dex TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;

pub const CREATE_POSITIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS positions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token_mint TEXT NOT NULL UNIQUE,
    token_symbol TEXT NOT NULL,
    entry_time TEXT NOT NULL,
    entry_price TEXT NOT NULL,
    entry_mc TEXT NOT NULL,
    amount TEXT NOT NULL,
    stop_loss TEXT NOT NULL,
    take_profit TEXT NOT NULL,
    status TEXT NOT NULL,
    exit_time TEXT,
    exit_price TEXT,
    exit_reason TEXT,
    pnl TEXT,
    pnl_pct REAL,
    hold_time_minutes INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;

pub const CREATE_WALLETS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS wallets (
    address TEXT PRIMARY KEY,
    label TEXT,
    smart_money_score REAL NOT NULL,
    risk_score REAL NOT NULL,
    is_tracked INTEGER NOT NULL DEFAULT 1,
    first_seen TEXT NOT NULL,
    last_active TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;

pub const CREATE_WALLET_METRICS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS wallet_metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_address TEXT NOT NULL,
    total_trades INTEGER NOT NULL,
    winning_trades INTEGER NOT NULL,
    losing_trades INTEGER NOT NULL,
    win_rate REAL NOT NULL,
    total_pnl TEXT NOT NULL,
    total_pnl_percentage REAL NOT NULL,
    avg_hold_time_seconds REAL NOT NULL,
    avg_profit_per_trade TEXT NOT NULL,
    largest_win TEXT NOT NULL,
    largest_loss TEXT NOT NULL,
    sharpe_ratio REAL,
    max_drawdown REAL NOT NULL,
    trades_last_24h INTEGER NOT NULL,
    trades_last_7d INTEGER NOT NULL,
    volume_24h TEXT NOT NULL,
    volume_7d TEXT NOT NULL,
    snapshot_time TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (wallet_address) REFERENCES wallets(address)
)
"#;

pub const CREATE_DAILY_STATS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS daily_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL UNIQUE,
    total_trades INTEGER NOT NULL DEFAULT 0,
    wins INTEGER NOT NULL DEFAULT 0,
    losses INTEGER NOT NULL DEFAULT 0,
    win_rate REAL NOT NULL DEFAULT 0,
    total_pnl TEXT NOT NULL DEFAULT '0',
    biggest_win TEXT NOT NULL DEFAULT '0',
    biggest_loss TEXT NOT NULL DEFAULT '0',
    avg_win TEXT NOT NULL DEFAULT '0',
    avg_loss TEXT NOT NULL DEFAULT '0',
    portfolio_value TEXT NOT NULL DEFAULT '0',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;

pub const CREATE_SIGNALS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS signals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token_mint TEXT NOT NULL,
    signal_type TEXT NOT NULL,
    confidence REAL NOT NULL,
    smart_wallets_count INTEGER NOT NULL,
    avg_smart_score REAL NOT NULL,
    total_volume TEXT NOT NULL,
    chart_action TEXT,
    chart_confidence REAL,
    chart_reason TEXT,
    executed INTEGER NOT NULL DEFAULT 0,
    execution_price TEXT,
    execution_time TEXT,
    detected_at TEXT NOT NULL DEFAULT (datetime('now'))
)
"#;
