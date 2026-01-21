# Telegram Notification Setup

Get real-time notifications about your trading bot's activity directly in Telegram!

## Features

- 🤖 Bot startup notifications
- 🟢 New position opened alerts
- ✅❌ Position closed notifications (wins/losses)
- 🎉 Big win celebrations ($5+ profit)
- 🔥 Ultra-high confidence signal alerts
- ⚠️ Scam detection warnings
- 📊 Periodic portfolio updates
- ♻️ Cycle completion summaries

## Setup Instructions

### Step 1: Create a Telegram Bot

1. Open Telegram and search for **@BotFather**
2. Send `/newbot` command
3. Follow the prompts to name your bot
4. Copy the **bot token** (looks like `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

### Step 2: Get Your Chat ID

**Option A: Use @userinfobot**
1. Search for **@userinfobot** on Telegram
2. Start a chat and it will send you your user ID
3. Copy the **ID** number

**Option B: Use your bot**
1. Start a chat with your new bot
2. Send any message to it
3. Visit: `https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates`
4. Look for `"chat":{"id":` in the response
5. Copy the **chat ID** number

### Step 3: Configure Environment Variables

1. Copy `.env.example` to `.env`:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` and set:
   ```bash
   TELEGRAM_ENABLED=true
   TELEGRAM_BOT_TOKEN=your_bot_token_here
   TELEGRAM_CHAT_ID=your_chat_id_here
   ```

### Step 4: Test It!

Run the bot:
```bash
cargo run --bin bot-enhanced
```

You should receive a test notification: "✅ Telegram Bot Connected!"

## Notification Examples

### Position Opened
```
🟢 NEW POSITION OPENED

🪙 Token: BONK
💵 Entry Price: $0.000012
💰 Amount: $10.00
📊 Confidence: 87%

🎯 Take Profit: $0.000024 (+100.0%)
🛑 Stop Loss: $0.000011 (-10.0%)

⏰ 14:23:45 UTC
```

### Position Closed (Win)
```
🟢 POSITION CLOSED - WIN

🪙 Token: BONK
💵 Entry: $0.000012
💵 Exit: $0.000018
📊 PnL: $5.00 (+50.0%)
⏱️ Hold Time: 120 min

✅ Nice trade!
```

### Big Win Celebration
```
🎉🎉🎉 BIG WIN! 🎉🎉🎉

💰 Profit: $15.50
📈 Gain: 155.0%
🪙 Token: WIF

Keep crushing it! 🚀🚀🚀
```

### Portfolio Update
```
📈 PORTFOLIO UPDATE

💰 Portfolio Value: $125.50
📊 Daily PnL: $25.50 (+25.5%)
🎯 Win Rate: 7/10 (70.0%)

📈 Biggest Win: $15.50
📉 Biggest Loss: $-2.00

💎 ROI: 25.5%
⏰ 2025-01-21 15:00:00 UTC
```

## Troubleshooting

### "Failed to send Telegram message"
- Check your bot token is correct
- Make sure you've started a chat with your bot
- Verify your chat ID is correct

### "TELEGRAM_BOT_TOKEN must be set"
- Ensure `.env` file exists in the project root
- Check `TELEGRAM_ENABLED=true` is set
- Verify the token variable name matches exactly

### No notifications received
- Confirm you've sent at least one message to your bot
- Try restarting the bot
- Check the bot logs for errors

## Privacy & Security

- Your bot token is like a password - keep it secret!
- Never commit `.env` to git (it's already in `.gitignore`)
- Only share your bot with trusted users
- You can revoke/regenerate tokens via @BotFather if compromised

## Advanced: Group Notifications

To send notifications to a group:

1. Add your bot to a Telegram group
2. Make the bot an admin (required to post)
3. Get the group chat ID:
   - Send a message in the group
   - Visit: `https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates`
   - Look for `"chat":{"id":-` (negative number for groups)
4. Use the group chat ID (including the minus sign) in your `.env`

## Disabling Notifications

To disable notifications without removing the configuration:

```bash
TELEGRAM_ENABLED=false
```

The bot will run normally but won't send any Telegram messages.
