use std::{env, time::Duration};

use anyhow::bail;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::logger::log_retry;

#[derive(Serialize)]
pub struct AiRequest {
    pub model: String,
    pub instructions: String,
    pub input: String,
}

impl AiRequest {
    pub fn new(instructions: String, input: String) -> Self {
        Self {
            model: "gpt-4o-mini".into(),
            instructions,
            input,
        }
    }
}

#[derive(Deserialize)]
pub struct AiReponse {
    output: Vec<ResponseContent>,
}

#[derive(Deserialize)]
struct ResponseContent {
    content: Vec<ContentLine>,
}

#[derive(Deserialize)]
struct ContentLine {
    text: String,
}

pub struct OpenAI {
    api_key: String,
    client: Client,
}

impl OpenAI {
    pub fn new(api_key: Option<String>) -> Self {
        let api_key = match api_key {
            Some(api_key) => api_key,
            None => Self::get_api_key().expect("OPENAI_API_KEY must be set"),
        };

        Self {
            api_key,
            client: Client::new(),
        }
    }

    fn get_api_key() -> anyhow::Result<String> {
        env::var("OPENAI_API_KEY").map_err(|_| anyhow::anyhow!("OPENAI_API_KEY must be set"))
    }

    pub async fn send(&self, body: AiRequest) -> anyhow::Result<String> {
        let mut retries = 0;
        let max_retries = 5;

        loop {
            let response = self
                .client
                .post("https://api.openai.com/v1/responses")
                .bearer_auth(self.api_key.clone())
                .json(&body)
                .send()
                .await;

            let response = match response {
                Ok(response) => response,
                Err(err) => {
                    retry(max_retries, &mut retries, &err.to_string()).await?;
                    continue;
                }
            };

            match response.status() {
                reqwest::StatusCode::OK => {
                    let response = response.json::<AiReponse>().await?;
                    return self.extract_translation_result(response);
                }
                _ => {
                    retry(max_retries, &mut retries, "TOO_MANY_REQUESTS").await?;
                    continue;
                }
            }
        }

        pub async fn retry(max_retries: u32, retries: &mut u32, err: &str) -> anyhow::Result<()> {
            if *retries > max_retries {
                bail!("Failed after {} retries", max_retries);
            }

            log_retry(*retries, max_retries, err);
            let wait = 2u64.pow(*retries) * 100; // exponential backoff: 100ms, 200ms, 400ms...
            let _ = sleep(Duration::from_millis(wait));
            *retries += 1;

            return Ok(());
        }
    }

    fn extract_translation_result(&self, response: AiReponse) -> anyhow::Result<String> {
        let text = &response.output[0].content[0].text;
        Ok(text.trim_matches('"').to_string())
    }
}
