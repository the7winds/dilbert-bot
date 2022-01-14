[![build badge](https://img.shields.io/github/workflow/status/the7winds/dilbert-bot/on-push)](https://github.com/the7winds/dilbert-bot/actions)
[![use rust](https://img.shields.io/github/languages/top/the7winds/dilbert-bot)](https://www.rust-lang.org)
[![try it in telegram](https://img.shields.io/badge/try%20it-in%20telegram-blue)](https://t.me/dilbertsearchbot)

# dilbert-bot

An inline telegram bot to fetch comics from [dilbert.com](https://dilbert.com).

### building

Just run `cargo build` with flags you prefer.

### running

Set `DILBERT_BOT_TELEGRAM_TOKEN` environment variable with your bot token.

Set `DILBERT_BOT_USE_CACHE=true` to use dummy cache to reduce number of requests to server.

You can run bot using `cargo run` or running binary itself.
