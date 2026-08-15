use crate::config::Config;
use crate::db::Storage;
use crate::i18n::get_text;
use crate::services::{AiService, FreelancehuntClient};
use std::sync::Arc;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use teloxide::utils::command::BotCommands;
use tracing::error;

pub type StateStorage = InMemStorage<DialogueState>;
pub type BotDialogue = Dialogue<DialogueState, StateStorage>;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub enum DialogueState {
    #[default]
    Idle,
    AwaitingBudgetDays {
        project_id: i64,
    },
    DraftReady {
        project_id: i64,
        budget: i64,
        days: i32,
        comment: String,
    },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Commands:")]
pub enum Command {
    #[command(description = "Start bot")]
    Start,
    #[command(description = "Show help")]
    Help,
    #[command(description = "Show bot status")]
    Status,
    #[command(description = "Cancel operation")]
    Cancel,
}

pub async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    config: Arc<Config>,
    storage: Storage,
    start_time: std::time::Instant,
) -> ResponseResult<()> {
    let lang = &config.language;

    match cmd {
        Command::Start => {
            let text = get_text(lang, "start_text");
            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Command::Help => {
            let text = get_text(lang, "help_text");
            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Command::Status => {
            let uptime_secs = start_time.elapsed().as_secs();
            let days = uptime_secs / 86400;
            let hours = (uptime_secs % 86400) / 3600;
            let mins = (uptime_secs % 3600) / 60;

            let uptime_str = format!("{}d {}h {}m", days, hours, mins);
            let count = storage.count_seen().await.unwrap_or(0);

            let tpl = get_text(lang, "status_text");
            let text = tpl
                .replace("{uptime}", &uptime_str)
                .replace("{last_parse}", "OK")
                .replace("{count}", &count.to_string());

            bot.send_message(msg.chat.id, text)
                .parse_mode(ParseMode::Html)
                .await?;
        }
        Command::Cancel => {
            bot.send_message(msg.chat.id, get_text(lang, "bid_canceled"))
                .await?;
        }
    }

    Ok(())
}

pub async fn handle_callback(
    bot: Bot,
    q: CallbackQuery,
    dialogue: BotDialogue,
    config: Arc<Config>,
    fh_client: FreelancehuntClient,
    ai_service: AiService,
) -> ResponseResult<()> {
    let lang = &config.language;
    let data = match q.data {
        Some(d) => d,
        None => return Ok(()),
    };

    bot.answer_callback_query(q.id.clone()).await?;

    let message = match q.message {
        Some(m) => m,
        None => return Ok(()),
    };

    if data.starts_with("analyze_") {
        let pid_str = &data["analyze_".len()..];
        if let Ok(pid) = pid_str.parse::<i64>() {
            let loading_msg = bot.send_message(message.chat().id, get_text(lang, "analyze_loading")).await?;

            match fh_client.get_project_detail(pid).await {
                Ok(proj) => match ai_service.analyze_project(&proj).await {
                    Ok(analysis) => {
                        let tpl = get_text(lang, "analyze_title");
                        let text = tpl.replace("{name}", &proj.name).replace("{analysis}", &analysis);

                        if let Err(e) = bot.edit_message_text(message.chat().id, loading_msg.id, &text)
                            .parse_mode(ParseMode::Html)
                            .await
                        {
                            error!("Failed HTML message edit for analysis: {}, retrying as plain text", e);
                            let _ = bot.edit_message_text(message.chat().id, loading_msg.id, &text).await;
                        }
                    }
                    Err(e) => {
                        error!("AI analysis error for #{}: {}", pid, e);
                        bot.edit_message_text(message.chat().id, loading_msg.id, get_text(lang, "analyze_error_msg"))
                            .await?;
                    }
                },
                Err(e) => {
                    error!("Project detail error for #{}: {}", pid, e);
                    bot.edit_message_text(message.chat().id, loading_msg.id, get_text(lang, "analyze_error_msg"))
                        .await?;
                }
            }
        }
    } else if data.starts_with("gen_bid_") {
        let pid_str = &data["gen_bid_".len()..];
        if let Ok(pid) = pid_str.parse::<i64>() {
            dialogue.update(DialogueState::AwaitingBudgetDays { project_id: pid }).await.ok();
            bot.send_message(message.chat().id, get_text(lang, "bid_enter_budget")).await?;
        }
    } else if data.starts_with("post_bid_") {
        if let Ok(state) = dialogue.get().await {
            if let Some(DialogueState::DraftReady { project_id, budget, days, comment }) = state {
                bot.send_message(message.chat().id, get_text(lang, "bid_sending")).await?;

                match fh_client.get_project_detail(project_id).await {
                    Ok(proj) => {
                        match fh_client.post_bid(&proj.url, budget, days, &comment).await {
                            Ok(true) => {
                                bot.send_message(message.chat().id, get_text(lang, "bid_success")).await?;
                                dialogue.exit().await.ok();
                            }
                            _ => {
                                bot.send_message(message.chat().id, get_text(lang, "bid_send_warn")).await?;
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to get project detail for post bid: {}", e);
                    }
                }
            }
        }
    } else if data == "cancel" {
        dialogue.exit().await.ok();
        bot.send_message(message.chat().id, get_text(lang, "bid_canceled")).await?;
    }

    Ok(())
}

pub async fn handle_input(
    bot: Bot,
    msg: Message,
    dialogue: BotDialogue,
    config: Arc<Config>,
    fh_client: FreelancehuntClient,
    ai_service: AiService,
) -> ResponseResult<()> {
    let lang = &config.language;
    let state = match dialogue.get().await {
        Ok(Some(s)) => s,
        _ => return Ok(()),
    };

    if let Some(text) = msg.text() {
        if text == "/cancel" {
            dialogue.exit().await.ok();
            bot.send_message(msg.chat.id, get_text(lang, "bid_canceled")).await?;
            return Ok(());
        }

        match state {
            DialogueState::AwaitingBudgetDays { project_id } => {
                let parts: Vec<&str> = text.split_whitespace().collect();
                if parts.len() != 2 {
                    bot.send_message(msg.chat.id, get_text(lang, "bid_format_error")).await?;
                    return Ok(());
                }

                let budget = match parts[0].parse::<i64>() {
                    Ok(b) => b,
                    Err(_) => {
                        bot.send_message(msg.chat.id, get_text(lang, "bid_format_error")).await?;
                        return Ok(());
                    }
                };

                let days = match parts[1].parse::<i32>() {
                    Ok(d) => d,
                    Err(_) => {
                        bot.send_message(msg.chat.id, get_text(lang, "bid_format_error")).await?;
                        return Ok(());
                    }
                };

                let loading_msg = bot.send_message(msg.chat.id, get_text(lang, "bid_generating")).await?;

                match fh_client.get_project_detail(project_id).await {
                    Ok(proj) => match ai_service.generate_proposal(&proj, budget, days).await {
                        Ok(proposal) => {
                            let draft_tpl = get_text(lang, "bid_draft");
                            let resp_text = draft_tpl
                                .replace("{budget}", &budget.to_string())
                                .replace("{currency}", &proj.budget.currency)
                                .replace("{days}", &days.to_string())
                                .replace("{text}", &proposal);

                            let keyboard = InlineKeyboardMarkup::new(vec![
                                vec![InlineKeyboardButton::callback(
                                    get_text(lang, "btn_send_bid"),
                                    format!("post_bid_{}", project_id),
                                )],
                                vec![InlineKeyboardButton::callback(
                                    get_text(lang, "btn_cancel"),
                                    "cancel".to_string(),
                                )],
                            ]);

                            if let Err(e) = bot.edit_message_text(msg.chat.id, loading_msg.id, &resp_text)
                                .parse_mode(ParseMode::Html)
                                .reply_markup(keyboard.clone())
                                .await
                            {
                                error!("Failed HTML edit for proposal draft: {}, retrying as plain text", e);
                                let _ = bot.edit_message_text(msg.chat.id, loading_msg.id, &resp_text)
                                    .reply_markup(keyboard)
                                    .await;
                            }

                            dialogue.update(DialogueState::DraftReady {
                                project_id,
                                budget,
                                days,
                                comment: proposal,
                            }).await.ok();
                        }
                        Err(e) => {
                            error!("Proposal generation error: {}", e);
                            bot.edit_message_text(msg.chat.id, loading_msg.id, "❌ Error generating proposal.")
                                .await?;
                        }
                    },
                    Err(e) => {
                        error!("Fetch project error: {}", e);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
