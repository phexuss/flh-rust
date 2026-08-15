use crate::config::Config;
use crate::i18n::get_text;
use crate::models::Project;
use anyhow::Result;
use std::sync::Arc;
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};
use tracing::{error, info};

pub struct Notifier {
    bot: Bot,
    config: Arc<Config>,
}

impl Notifier {
    pub fn new(bot: Bot, config: Arc<Config>) -> Self {
        Self { bot, config }
    }

    pub async fn send_project(&self, project: &Project) -> Result<bool> {
        if self.config.telegram_chat_id == 0 {
            error!("TELEGRAM_CHAT_ID is 0! Cannot send notification.");
            return Ok(false);
        }

        let text = self.format_project_message(project);
        let keyboard = self.create_keyboard(project);

        let chat_id = ChatId(self.config.telegram_chat_id);

        match self.bot
            .send_message(chat_id, text)
            .parse_mode(ParseMode::Html)
            .reply_markup(keyboard)
            .await
        {
            Ok(_) => {
                info!("Sent project notification #{}: {}", project.id, project.name);
                Ok(true)
            }
            Err(e) => {
                error!("Failed to send project notification #{}: {}", project.id, e);
                Ok(false)
            }
        }
    }

    fn format_project_message(&self, project: &Project) -> String {
        let lang = &self.config.language;
        let mut text = String::new();

        text.push_str(get_text(lang, "notify_new_project"));
        text.push_str("\n\n");

        let title_tpl = get_text(lang, "notify_title");
        text.push_str(&title_tpl.replace("{name}", &html_escape(&project.name)));
        text.push('\n');

        let budget_tpl = get_text(lang, "notify_budget");
        text.push_str(&budget_tpl.replace("{budget}", &project.budget_text()));
        text.push('\n');

        if !project.skills.is_empty() {
            let skills_tpl = get_text(lang, "notify_skills");
            text.push_str(&skills_tpl.replace("{skills}", &html_escape(&project.skills_text())));
            text.push('\n');
        }

        if !project.employer.login.is_empty() {
            let emp_tpl = get_text(lang, "notify_employer");
            let emp_name = format!("{} (@{})", project.employer.first_name, project.employer.login);
            text.push_str(&emp_tpl.replace("{employer}", &html_escape(&emp_name)));
            text.push('\n');
        }

        if let Some(pub_at) = &project.published_at {
            let pub_tpl = get_text(lang, "notify_published");
            text.push_str(&pub_tpl.replace("{published}", pub_at));
            text.push('\n');
        }

        let bids_tpl = get_text(lang, "notify_bids");
        text.push_str(&bids_tpl.replace("{bids}", &project.bid_count.to_string()));
        text.push_str("\n\n");

        text.push_str(get_text(lang, "notify_desc"));

        let mut desc = project.description.trim().to_string();
        if desc.len() > 800 {
            desc.truncate(800);
            desc.push_str("...");
        }

        text.push_str(&html_escape(&desc));
        text
    }

    fn create_keyboard(&self, project: &Project) -> InlineKeyboardMarkup {
        let lang = &self.config.language;

        let btn_open = InlineKeyboardButton::url(
            get_text(lang, "btn_open_project"),
            url::Url::parse(&project.url).unwrap_or_else(|_| url::Url::parse("https://freelancehunt.com").unwrap()),
        );

        let btn_gen = InlineKeyboardButton::callback(
            get_text(lang, "btn_gen_bid"),
            format!("gen_bid_{}", project.id),
        );

        let btn_analyze = InlineKeyboardButton::callback(
            get_text(lang, "btn_analyze"),
            format!("analyze_{}", project.id),
        );

        InlineKeyboardMarkup::new(vec![
            vec![btn_open],
            vec![btn_gen, btn_analyze],
        ])
    }
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
