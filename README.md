# Freelance Sniper Bot (Rust Edition)

High-performance, low-memory Telegram bot for monitoring [Freelancehunt](https://freelancehunt.com) projects with AI proposal generation (Gemini / Groq / OpenRouter), rewritten in Rust.

## Features

- **Ultra-low Memory Usage**: Takes ~15-20 MB RAM in production (vs 100-500 MB in Python).
- **Fast Polling**: Periodically polls new projects matching target categories.
- **SQLite Deduplication**: Async SQLite storage via `sqlx`.
- **Telegram Interface**: Powered by `teloxide` v0.13 with inline keyboards and dialogue flow for submitting bids.
- **AI Integrations**: Gemini AI / Groq / OpenRouter support for generating custom proposals and project analysis.
- **Direct Form Bids**: Submits proposal bids via HTTP forms without browser (no Chromium memory overhead).

## Quick Start

### 1. Configure Environment

Copy `.env.example` to `.env` and fill in credentials:

```bash
cp .env.example .env
```

### 2. Build & Run locally

```bash
cargo run --release
```

### 3. Run with Docker

```bash
docker-compose up -d --build
```
