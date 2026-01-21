#!/bin/bash

set -e

echo "🚀 Solana Trading Bot Setup"
echo "============================"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
else
    echo "✅ Rust installed: $(rustc --version)"
fi

# Check Docker (optional)
if command -v docker &> /dev/null; then
    echo "✅ Docker installed: $(docker --version)"
else
    echo "⚠️  Docker not found (optional, but recommended for database)"
fi

# Check PostgreSQL
if command -v psql &> /dev/null; then
    echo "✅ PostgreSQL client installed"
else
    echo "⚠️  PostgreSQL client not found"
fi

echo ""
echo "📝 Setting up configuration..."

# Copy .env if it doesn't exist
if [ ! -f .env ]; then
    cp .env.example .env
    echo "✅ Created .env file from template"
    echo "⚠️  Please edit .env and add your configuration"
else
    echo "✅ .env file already exists"
fi

echo ""
echo "🗄️  Setting up database..."

# Offer to start Docker containers
if command -v docker &> /dev/null; then
    read -p "Start PostgreSQL and Redis with Docker? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Starting PostgreSQL..."
        docker run -d --name trading-postgres \
          -e POSTGRES_PASSWORD=password \
          -e POSTGRES_DB=trading_bot \
          -p 5432:5432 \
          postgres:14 || echo "PostgreSQL container may already exist"

        echo "Starting Redis..."
        docker run -d --name trading-redis \
          -p 6379:6379 \
          redis:6 || echo "Redis container may already exist"

        echo "✅ Database containers started"
        sleep 3
    fi
fi

echo ""
echo "🔧 Building project..."
cargo build --release

echo ""
echo "✅ Setup complete!"
echo ""
echo "📚 Next steps:"
echo "1. Edit .env file with your configuration:"
echo "   - Add your Solana wallet private key"
echo "   - Add Helius API key (get free at helius.dev)"
echo "   - Configure risk limits"
echo ""
echo "2. Run database migrations (when implemented):"
echo "   sqlx migrate run"
echo ""
echo "3. Test in dry-run mode:"
echo "   cargo run --release --bin bot"
echo ""
echo "4. When ready for live trading, set TRADING_ENABLED=true in .env"
echo ""
echo "📖 Read IMPLEMENTATION_ROADMAP.md for development guide"
echo "📖 Read README.md for usage instructions"
echo ""
echo "⚠️  IMPORTANT: Start with paper trading (TRADING_ENABLED=false)"
echo "             and test for at least 1 week before live trading!"
