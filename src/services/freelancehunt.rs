use crate::config::Config;
use crate::models::Project;
use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, CONTENT_TYPE, USER_AGENT};
use reqwest::Client;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct FreelancehuntClient {
    client: Client,
    config: Arc<Config>,
}

impl FreelancehuntClient {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .cookie_store(true)
            .build()?;

        Ok(Self { client, config })
    }

    fn default_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_str(&self.config.language).unwrap_or(HeaderValue::from_static("uk")));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        if !self.config.freelancehunt_api_token.is_empty() {
            let auth = format!("Bearer {}", self.config.freelancehunt_api_token);
            if let Ok(hv) = HeaderValue::from_str(&auth) {
                headers.insert("Authorization", hv);
            }
        } else {
            warn!("FREELANCEHUNT_API_TOKEN is missing!");
        }

        headers
    }

    pub async fn get_projects(&self, skill_ids: &[i32], page: u32) -> Result<Vec<Project>> {
        if skill_ids.len() > 1 {
            return self.get_projects_multi_skill(skill_ids, page).await;
        }

        let mut url = format!("{}projects?page[number]={}", self.config.freelancehunt_base_url, page);
        if let Some(&sid) = skill_ids.first() {
            url.push_str(&format!("&filter[skill_id]={}", sid));
        }

        let res = self.client
            .get(&url)
            .headers(self.default_headers())
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow!("Freelancehunt API returned status {}", res.status()));
        }

        let json: Value = res.json().await?;
        let data = json.get("data").and_then(|v| v.as_array()).cloned().unwrap_or_default();

        let projects = data.iter().map(Project::from_api).collect();
        Ok(projects)
    }

    async fn get_projects_multi_skill(&self, skill_ids: &[i32], page: u32) -> Result<Vec<Project>> {
        let mut seen_ids = HashSet::new();
        let mut projects = Vec::new();

        for &sid in skill_ids {
            let url = format!(
                "{}projects?page[number]={}&filter[skill_id]={}",
                self.config.freelancehunt_base_url, page, sid
            );

            match self.client.get(&url).headers(self.default_headers()).send().await {
                Ok(res) if res.status().is_success() => {
                    if let Ok(json) = res.json::<Value>().await {
                        if let Some(data) = json.get("data").and_then(|v| v.as_array()) {
                            for item in data {
                                let p = Project::from_api(item);
                                if seen_ids.insert(p.id) {
                                    projects.push(p);
                                }
                            }
                        }
                    }
                }
                Ok(res) => {
                    warn!("Failed fetching skill_id {}: status {}", sid, res.status());
                }
                Err(e) => {
                    error!("Error fetching skill_id {}: {}", sid, e);
                }
            }
        }

        info!("Fetched {} unique projects for {} skills", projects.len(), skill_ids.len());
        Ok(projects)
    }

    pub async fn get_project_detail(&self, project_id: i64) -> Result<Project> {
        let url = format!("{}projects/{}", self.config.freelancehunt_base_url, project_id);
        let res = self.client
            .get(&url)
            .headers(self.default_headers())
            .send()
            .await?;

        let json: Value = res.json().await?;
        let data = json.get("data").cloned().unwrap_or(Value::Null);

        Ok(Project::from_api(&data))
    }

    /// Direct HTTP Form Bid submission (no headless browser required!)
    pub async fn post_bid(&self, project_url: &str, budget: i64, days: i32, comment: &str) -> Result<bool> {
        if self.config.freelancehunt_session_cookie.is_empty() {
            error!("FREELANCEHUNT_SESSION_COOKIE is empty. Cannot post bid.");
            return Ok(false);
        }

        let cookie_str = if self.config.freelancehunt_session_cookie.contains('=') {
            self.config.freelancehunt_session_cookie.clone()
        } else {
            format!("PHPSESSID={}", self.config.freelancehunt_session_cookie)
        };

        let mut web_headers = HeaderMap::new();
        web_headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
        web_headers.insert("Cookie", HeaderValue::from_str(&cookie_str)?);

        info!("Fetching project page HTML for CSRF token: {}", project_url);
        let resp = self.client.get(project_url).headers(web_headers.clone()).send().await?;

        if !resp.status().is_success() {
            error!("Failed to fetch project page, status: {}", resp.status());
            return Ok(false);
        }

        let html_text = resp.text().await?;

        // Scoped block to ensure `scraper::Html` is dropped before any `.await`
        let (target_form_action, csrf_token) = {
            let document = Html::parse_document(&html_text);

            let form_selector = Selector::parse("form").unwrap();
            let textarea_selector = Selector::parse("textarea[name='comment'], textarea[name='message']").unwrap();
            let input_selector = Selector::parse("input").unwrap();

            let mut form_action = project_url.to_string();
            let mut token: Option<String> = None;
            let mut form_found = false;

            for form in document.select(&form_selector) {
                if form.select(&textarea_selector).next().is_some() {
                    form_found = true;
                    if let Some(action) = form.value().attr("action") {
                        if action.starts_with("http") {
                            form_action = action.to_string();
                        } else if action.starts_with('/') {
                            if let Ok(base_url) = url::Url::parse(project_url) {
                                if let Ok(joined) = base_url.join(action) {
                                    form_action = joined.to_string();
                                }
                            }
                        }
                    }

                    for input in form.select(&input_selector) {
                        let name = input.value().attr("name").unwrap_or("");
                        if name == "_token" || name == "csrf_token" || name == "_csrf" {
                            if let Some(val) = input.value().attr("value") {
                                token = Some(val.to_string());
                            }
                        }
                    }
                    break;
                }
            }

            if !form_found {
                warn!("Bid form not found in project page HTML");
            }

            (form_action, token)
        };

        info!("Submitting bid (budget: {}, days: {}) to {}", budget, days, target_form_action);

        let mut params = std::collections::HashMap::new();
        params.insert("amount", budget.to_string());
        params.insert("days", days.to_string());
        params.insert("comment", comment.to_string());
        params.insert("message", comment.to_string());

        if let Some(token) = &csrf_token {
            params.insert("_token", token.clone());
        }

        let mut post_headers = web_headers.clone();
        post_headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
        post_headers.insert("Referer", HeaderValue::from_str(project_url)?);

        let post_resp = self.client
            .post(&target_form_action)
            .headers(post_headers)
            .form(&params)
            .send()
            .await?;

        let post_status = post_resp.status();
        let post_body = post_resp.text().await?;

        // Scoped block for response error check
        let has_error_alert = {
            let post_doc = Html::parse_document(&post_body);
            let error_selector = Selector::parse(".alert-danger, .text-danger, .error-message").unwrap();
            if let Some(err_el) = post_doc.select(&error_selector).next() {
                let err_text = err_el.text().collect::<Vec<_>>().join(" ");
                warn!("Bid response contain error message: {}", err_text.trim());
                true
            } else {
                false
            }
        };

        if has_error_alert {
            return Ok(false);
        }

        if post_status.is_success() || post_status.is_redirection() {
            info!("✅ Bid posted successfully!");
            Ok(true)
        } else {
            error!("Bid submission failed with status code {}", post_status);
            Ok(false)
        }
    }
}
