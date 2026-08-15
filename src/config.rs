use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub freelancehunt_api_token: String,
    pub freelancehunt_base_url: String,
    pub freelancehunt_session_cookie: String,
    pub telegram_bot_token: String,
    pub telegram_chat_id: i64,
    pub ai_provider: String,
    pub gemini_api_key: String,
    pub groq_api_key: String,
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub parse_interval_minutes: u64,
    pub target_skill_ids: Vec<i32>,
    pub language: String,
    pub db_path: PathBuf,
    pub log_path: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        let freelancehunt_api_token = env::var("FREELANCEHUNT_API_TOKEN").unwrap_or_default();
        let freelancehunt_base_url = env::var("FREELANCEHUNT_BASE_URL")
            .unwrap_or_else(|_| "https://api.freelancehunt.com/v2/".to_string());
        let freelancehunt_session_cookie = env::var("FREELANCEHUNT_SESSION_COOKIE").unwrap_or_default();

        let telegram_bot_token = env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default();
        let telegram_chat_id = env::var("TELEGRAM_CHAT_ID")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let ai_provider = env::var("AI_PROVIDER")
            .unwrap_or_else(|_| "gemini".to_string())
            .to_lowercase();
        let gemini_api_key = env::var("GEMINI_API_KEY").unwrap_or_default();
        let groq_api_key = env::var("GROQ_API_KEY").unwrap_or_default();
        let openrouter_api_key = env::var("OPENROUTER_API_KEY").unwrap_or_default();
        let openrouter_model = env::var("OPENROUTER_MODEL")
            .unwrap_or_else(|_| "google/gemini-2.5-flash".to_string());

        let parse_interval_minutes = env::var("PARSE_INTERVAL_MINUTES")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(7);

        let target_skill_ids = env::var("TARGET_SKILL_IDS")
            .unwrap_or_else(|_| "1,2,86,88,96,99,166".to_string())
            .split(',')
            .filter_map(|s| s.trim().parse::<i32>().ok())
            .collect();

        let language = env::var("LANGUAGE").unwrap_or_else(|_| "uk".to_string());

        let db_path = PathBuf::from(env::var("DB_PATH").unwrap_or_else(|_| "data/bot.db".to_string()));
        let log_path = PathBuf::from(env::var("LOG_PATH").unwrap_or_else(|_| "logs/bot.log".to_string()));

        Self {
            freelancehunt_api_token,
            freelancehunt_base_url,
            freelancehunt_session_cookie,
            telegram_bot_token,
            telegram_chat_id,
            ai_provider,
            gemini_api_key,
            groq_api_key,
            openrouter_api_key,
            openrouter_model,
            parse_interval_minutes,
            target_skill_ids,
            language,
            db_path,
            log_path,
        }
    }
}
