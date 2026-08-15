use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Budget {
    pub amount: Option<i64>,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Employer {
    pub id: i64,
    pub login: String,
    pub first_name: String,
    pub last_name: String,
    pub rating: f64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Skill {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub budget: Budget,
    pub skills: Vec<Skill>,
    pub employer: Employer,
    pub url: String,
    pub published_at: Option<String>,
    pub status_name: String,
    pub bid_count: i32,
}

impl Project {
    pub fn skills_text(&self) -> String {
        self.skills
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn budget_text(&self) -> String {
        match self.budget.amount {
            Some(amount) => format!("{} {}", amount, self.budget.currency),
            None => "Не указан".to_string(),
        }
    }

    pub fn from_api(data: &Value) -> Self {
        let id = data.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let attrs = data.get("attributes").cloned().unwrap_or(Value::Null);

        let budget_raw = attrs.get("budget");
        let amount = budget_raw.and_then(|b| b.get("amount")).and_then(|v| v.as_i64());
        let currency = budget_raw
            .and_then(|b| b.get("currency"))
            .and_then(|v| v.as_str())
            .unwrap_or("UAH")
            .to_string();

        let employer_raw = attrs.get("employer");
        let employer = Employer {
            id: employer_raw.and_then(|e| e.get("id")).and_then(|v| v.as_i64()).unwrap_or(0),
            login: employer_raw.and_then(|e| e.get("login")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
            first_name: employer_raw.and_then(|e| e.get("first_name")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
            last_name: employer_raw.and_then(|e| e.get("last_name")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
            rating: employer_raw.and_then(|e| e.get("rating")).and_then(|v| v.as_f64()).unwrap_or(0.0),
            url: employer_raw.and_then(|e| e.get("self")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
        };

        let skills = attrs
            .get("skills")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .map(|item| Skill {
                        id: item.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                        name: item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let published_at = attrs
            .get("published_at")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut project_url = String::new();
        if let Some(links) = data.get("links") {
            if let Some(self_link) = links.get("self") {
                if let Some(web) = self_link.get("web").and_then(|v| v.as_str()) {
                    project_url = web.to_string();
                } else if let Some(api) = self_link.get("api").and_then(|v| v.as_str()) {
                    project_url = api.to_string();
                } else if let Some(str_link) = self_link.as_str() {
                    project_url = str_link.to_string();
                }
            }
        }

        if project_url.is_empty() || project_url.contains("api.freelancehunt") {
            project_url = format!("https://freelancehunt.com/project/{}/view", id);
        }

        let name = attrs.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let description = attrs
            .get("description")
            .and_then(|v| v.as_str())
            .or_else(|| attrs.get("description_text").and_then(|v| v.as_str()))
            .unwrap_or("")
            .to_string();

        let status_name = attrs
            .get("status")
            .and_then(|s| s.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let bid_count = attrs.get("bid_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

        Self {
            id,
            name,
            description,
            budget: Budget { amount, currency },
            skills,
            employer,
            url: project_url,
            published_at,
            status_name,
            bid_count,
        }
    }
}
