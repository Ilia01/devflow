use crate::models::ticket::JiraTicket;
use anyhow::{Context, Result};
use reqwest::Client;

pub struct JiraClient {
    client: Client,
    base_url: String,
    email: String,
    api_token: String,
}

impl JiraClient {
    pub fn new(base_url: String, email: String, api_token: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            email,
            api_token,
        }
    }

    pub async fn get_ticket(&self, ticket_id: &str) -> Result<JiraTicket> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, ticket_id);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.api_token))
            .send()
            .await
            .context("Failed to send request to Jira")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Jira API error ({}): {}", status, text);
        }

        let ticket = response
            .json::<JiraTicket>()
            .await
            .context("Failed to parse Jira response")?;

        Ok(ticket)
    }

    pub async fn update_status(&self, ticket_id: &str, transition_name: &str) -> Result<()> {
        let transitions_url = format!(
            "{}/rest/api/3/issue/{}/transitions",
            self.base_url, ticket_id
        );

        let transitions_response = self
            .client
            .get(&transitions_url)
            .basic_auth(&self.email, Some(&self.api_token))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        let transitions = transitions_response["transitions"]
            .as_array()
            .context("No transitions found")?;

        let transition_id = transitions
            .iter()
            .find(|t| t["name"].as_str() == Some(transition_name))
            .and_then(|t| t["id"].as_str())
            .context(format!("Transition '{}' not found", transition_name))?;

        let body = serde_json::json!({
            "transition": {
                "id": transition_id
            }
        });

        let response = self
            .client
            .post(&transitions_url)
            .basic_auth(&self.email, Some(&self.api_token))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to update status: {}", response.status());
        }

        Ok(())
    }
}
