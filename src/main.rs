mod bot;
mod config;
mod db;
mod i18n;
mod jobs;
mod models;
mod services;

use bot::{handle_callback, handle_command, handle_input, BotDialogue, Command, DialogueState, Notifier};
use config::Config;
use db::Storage;
use i18n::get_text;
use jobs::{spawn_cleanup_job, spawn_parser_job};
use services::{AiService, FreelancehuntClient};
use std::sync::Arc;
use std::time::Instant;
use teloxide::dispatching::dialogue::{enter, InMemStorage};
use teloxide::prelude::*;
use teloxide::types::ParseMode;
use tracing::{error, info};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::from_env());

    // Setup logging
    if let Some(parent) = config.log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file_appender = tracing_appender::rolling::daily(
        config.log_path.parent().unwrap_or(std::path::Path::new("logs")),
        config.log_path.file_name().unwrap_or(std::ffi::OsStr::new("bot.log")),
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::new("info"))
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    info!("Starting Freelance Sniper Bot (Rust Edition)...");

    if config.telegram_bot_token.is_empty() {
        error!("TELEGRAM_BOT_TOKEN is not set in .env!");
        return Ok(());
    }

    // Initialize Storage
    let storage = Storage::init(&config.db_path).await?;

    // Initialize Services
    let fh_client = FreelancehuntClient::new(config.clone())?;
    let ai_service = AiService::new(config.clone());

    // Telegram Bot
    let bot = Bot::new(&config.telegram_bot_token);
    let notifier = Notifier::new(bot.clone(), config.clone());

    // Spawn periodic background tasks
    spawn_parser_job(config.clone(), fh_client.clone(), storage.clone(), notifier);
    spawn_cleanup_job(storage.clone());

    // Send Admin Startup Notification
    if config.telegram_chat_id != 0 {
        let tpl = get_text(&config.language, "bot_started_admin");
        let msg_text = tpl.replace("{interval}", &config.parse_interval_minutes.to_string());
        let _ = bot
            .send_message(ChatId(config.telegram_chat_id), msg_text)
            .parse_mode(ParseMode::Html)
            .await;
    }

    let start_time = Instant::now();
    let dialogue_storage = InMemStorage::<DialogueState>::new();

    let config_cmd = config.clone();
    let storage_cmd = storage.clone();

    let config_cb = config.clone();
    let fh_cb = fh_client.clone();
    let ai_cb = ai_service.clone();

    let config_inp = config.clone();
    let fh_inp = fh_client.clone();
    let ai_inp = ai_service.clone();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .filter_command::<Command>()
                .endpoint(move |b: Bot, m: Message, c: Command| {
                    let cfg = config_cmd.clone();
                    let st = storage_cmd.clone();
                    async move { handle_command(b, m, c, cfg, st, start_time).await }
                }),
        )
        .branch(
            enter::<Update, InMemStorage<DialogueState>, DialogueState, _>()
                .branch(
                    Update::filter_callback_query().endpoint(
                        move |b: Bot, q: CallbackQuery, d: BotDialogue| {
                            let cfg = config_cb.clone();
                            let fh = fh_cb.clone();
                            let ai = ai_cb.clone();
                            async move { handle_callback(b, q, d, cfg, fh, ai).await }
                        },
                    ),
                )
                .branch(
                    Update::filter_message().endpoint(
                        move |b: Bot, m: Message, d: BotDialogue| {
                            let cfg = config_inp.clone();
                            let fh = fh_inp.clone();
                            let ai = ai_inp.clone();
                            async move { handle_input(b, m, d, cfg, fh, ai).await }
                        },
                    ),
                ),
        );

    info!("Bot is running and polling for updates...");

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![dialogue_storage])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
