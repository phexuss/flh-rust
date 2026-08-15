use crate::config::Config;
use crate::models::Project;
use anyhow::{anyhow, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone)]
pub struct AiService {
    client: Client,
    config: Arc<Config>,
}

impl AiService {
    pub fn new(config: Arc<Config>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self { client, config }
    }

    pub async fn generate_proposal(&self, project: &Project, budget: i64, days: i32) -> Result<String> {
        let lang_name = if self.config.language.to_lowercase().starts_with("uk") {
            "украінською"
        } else {
            "русском"
        };

        let prompt = format!(
            "Ты опытный фрилансер-разработчик. Напиши короткий, убедительный и профессиональный отклик на проект.\n\
            Пиши на {} языке.\n\n\
            Детали проекта:\n\
            Название: {}\n\
            Бюджет: {} {}\n\
            Предлагаемый срок: {} дн.\n\
            Навыки: {}\n\
            Описание:\n{}\n\n\
            Правила отклика:\n\
            1. Будь вежлив, приветлив и сразу переходи к делу.\n\
            2. Покажи понимание задачи и предложи конкретное решение или стеки.\n\
            3. Без воды и клише («Я профессионал с 10 годами опыта»). Не пиши приветствия вроде «Здравствуйте, ув. Заказчик» слишком длинно.\n\
            4. Длина: 3-6 предложений.",
            lang_name,
            project.name,
            budget,
            project.budget.currency,
            days,
            project.skills_text(),
            project.description
        );

        self.call_llm(&prompt).await
    }

    pub async fn analyze_project(&self, project: &Project) -> Result<String> {
        let lang_name = if self.config.language.to_lowercase().starts_with("uk") {
            "украінською"
        } else {
            "русском"
        };

        let prompt = format!(
            "Проанализируй ТЗ проекта для фрилансера. Пиши на {} языке.\n\n\
            Детали проекта:\n\
            Название: {}\n\
            Бюджет: {}\n\
            Навыки: {}\n\
            Описание:\n{}\n\n\
            Дай ответ по пунктам:\n\
            1. 🎯 **Суть задачи** (1-2 предложения)\n\
            2. 🛠 **Стек / Технологии**\n\
            3. ⚖️ **Оценка бюджета и сроков** (адекватен ли бюджет? Сколько примерно нужно времени?)\n\
            4. ⚠️ **Подводные камни и риски** (неясности в ТЗ, сложные моменты)\n\
            5. 💡 **Совет для отклика** (на что сделать акцент)",
            lang_name,
            project.name,
            project.budget_text(),
            project.skills_text(),
            project.description
        );

        self.call_llm(&prompt).await
    }

    async fn call_llm(&self, prompt: &str) -> Result<String> {
        match self.config.ai_provider.as_str() {
            "groq" => self.call_groq(prompt).await,
            "openrouter" => self.call_openrouter(prompt).await,
            _ => self.call_gemini(prompt).await,
        }
    }

    async fn call_gemini(&self, prompt: &str) -> Result<String> {
        let key = &self.config.gemini_api_key;
        if key.is_empty() {
            return Err(anyhow!("GEMINI_API_KEY is not set"));
        }

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
            key
        );

        let body = json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }]
        });

        let resp = self.client.post(&url).json(&body).send().await?;
        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Gemini API error: {}", err_text));
        }

        let json: Value = resp.json().await?;
        let text = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response structure from Gemini API"))?;

        Ok(text.trim().to_string())
    }

    async fn call_groq(&self, prompt: &str) -> Result<String> {
        let key = &self.config.groq_api_key;
        if key.is_empty() {
            return Err(anyhow!("GROQ_API_KEY is not set"));
        }

        let url = "https://api.groq.com/openai/v1/chat/completions";

        let body = json!({
            "model": "llama-3.3-70b-versatile",
            "messages": [
                { "role": "user", "content": prompt }
            ]
        });

        let resp = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", key))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Groq API error: {}", err_text));
        }

        let json: Value = resp.json().await?;
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response structure from Groq API"))?;

        Ok(text.trim().to_string())
    }

    async fn call_openrouter(&self, prompt: &str) -> Result<String> {
        let key = &self.config.openrouter_api_key;
        if key.is_empty() {
            return Err(anyhow!("OPENROUTER_API_KEY is not set"));
        }

        let url = "https://openrouter.ai/api/v1/chat/completions";

        let body = json!({
            "model": self.config.openrouter_model,
            "messages": [
                { "role": "user", "content": prompt }
            ]
        });

        let resp = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", key))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let err_text = resp.text().await.unwrap_or_default();
            return Err(anyhow!("OpenRouter API error: {}", err_text));
        }

        let json: Value = resp.json().await?;
        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response structure from OpenRouter API"))?;

        Ok(text.trim().to_string())
    }
}
