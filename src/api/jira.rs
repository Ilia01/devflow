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

    pub async fn search_tickets(&self, project_key: &str) -> Result<Vec<crate::models::ticket::JiraTicket>> {
        let jql = format!("assignee = currentUser() AND project = {}", project_key);
        let url = format!("{}/rest/api/3/search", self.base_url);

        let body = serde_json::json!({
            "jql": jql,
            "fields": ["summary", "status", "assignee"],
            "maxResults": 50
        });

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.email, Some(&self.api_token))
            .json(&body)
            .send()
            .await
            .context("Failed to search tickets")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Jira search API error ({}): {}", status, text);
        }

        let result: serde_json::Value = response.json().await.context("Failed to parse search response")?;
        let issues = result["issues"].as_array().context("No issues in response")?;

        let tickets: Vec<crate::models::ticket::JiraTicket> = issues
            .iter()
            .filter_map(|issue| serde_json::from_value(issue.clone()).ok())
            .collect();

        Ok(tickets)
    }

    pub async fn search_with_jql(&self, jql: &str, max_results: u32) -> Result<Vec<crate::models::ticket::JiraTicket>> {
        let url = format!("{}/rest/api/3/search", self.base_url);

        let body = serde_json::json!({
            "jql": jql,
            "fields": ["summary", "status", "assignee"],
            "maxResults": max_results
        });

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.email, Some(&self.api_token))
            .json(&body)
            .send()
            .await
            .context("Failed to search tickets")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Jira search API error ({}): {}", status, text);
        }

        let result: serde_json::Value = response.json().await.context("Failed to parse search response")?;
        let issues = result["issues"].as_array().context("No issues in response")?;

        let tickets: Vec<crate::models::ticket::JiraTicket> = issues
            .iter()
            .filter_map(|issue| serde_json::from_value(issue.clone()).ok())
            .collect();

        Ok(tickets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jira_client_creation() {
        let client = JiraClient::new(
            "https://jira.example.com".to_string(),
            "test@example.com".to_string(),
            "test-token".to_string(),
        );
        assert_eq!(client.base_url, "https://jira.example.com");
        assert_eq!(client.email, "test@example.com");
        assert_eq!(client.api_token, "test-token");
    }

    #[tokio::test]
    async fn test_search_tickets_success() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = serde_json::json!({
            "issues": [
                {
                    "key": "WAB-123",
                    "fields": {
                        "summary": "Test ticket 1",
                        "status": {
                            "name": "In Progress"
                        }
                    }
                },
                {
                    "key": "WAB-124",
                    "fields": {
                        "summary": "Test ticket 2",
                        "status": {
                            "name": "To Do"
                        }
                    }
                }
            ]
        });

        let _m = server
            .mock("POST", "/rest/api/3/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let client = JiraClient::new(
            server.url(),
            "test@example.com".to_string(),
            "test-token".to_string(),
        );

        let tickets = client.search_tickets("WAB").await.unwrap();

        assert_eq!(tickets.len(), 2);
        assert_eq!(tickets[0].key, "WAB-123");
        assert_eq!(tickets[0].fields.summary, "Test ticket 1");
        assert_eq!(tickets[0].fields.status.name, "In Progress");
        assert_eq!(tickets[1].key, "WAB-124");
        assert_eq!(tickets[1].fields.summary, "Test ticket 2");
    }

    #[tokio::test]
    async fn test_search_tickets_empty_results() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = serde_json::json!({
            "issues": []
        });

        let _m = server
            .mock("POST", "/rest/api/3/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let client = JiraClient::new(
            server.url(),
            "test@example.com".to_string(),
            "test-token".to_string(),
        );

        let tickets = client.search_tickets("WAB").await.unwrap();
        assert_eq!(tickets.len(), 0);
    }

    #[tokio::test]
    async fn test_search_tickets_api_error() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/rest/api/3/search")
            .with_status(401)
            .with_body("Unauthorized")
            .create_async()
            .await;

        let client = JiraClient::new(
            server.url(),
            "test@example.com".to_string(),
            "invalid-token".to_string(),
        );

        let result = client.search_tickets("WAB").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Jira search API error"));
    }

    #[tokio::test]
    async fn test_search_tickets_invalid_json() {
        let mut server = mockito::Server::new_async().await;

        let _m = server
            .mock("POST", "/rest/api/3/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("invalid json")
            .create_async()
            .await;

        let client = JiraClient::new(
            server.url(),
            "test@example.com".to_string(),
            "test-token".to_string(),
        );

        let result = client.search_tickets("WAB").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse search response"));
    }

    #[tokio::test]
    async fn test_search_tickets_missing_issues_field() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = serde_json::json!({
            "no_issues_field": []
        });

        let _m = server
            .mock("POST", "/rest/api/3/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let client = JiraClient::new(
            server.url(),
            "test@example.com".to_string(),
            "test-token".to_string(),
        );

        let result = client.search_tickets("WAB").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No issues in response"));
    }

    #[tokio::test]
    async fn test_search_with_jql_success() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = serde_json::json!({
            "issues": [
                {
                    "key": "WAB-100",
                    "fields": {
                        "summary": "Login bug fix",
                        "status": {
                            "name": "To Do"
                        }
                    }
                }
            ]
        });

        let _m = server
            .mock("POST", "/rest/api/3/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let client = JiraClient::new(
            server.url(),
            "test@example.com".to_string(),
            "test-token".to_string(),
        );

        let tickets = client.search_with_jql("summary ~ \"login\"", 10).await.unwrap();

        assert_eq!(tickets.len(), 1);
        assert_eq!(tickets[0].key, "WAB-100");
        assert_eq!(tickets[0].fields.summary, "Login bug fix");
    }

    #[tokio::test]
    async fn test_search_with_jql_respects_limit() {
        let mut server = mockito::Server::new_async().await;

        let mock_response = serde_json::json!({
            "issues": [
                {
                    "key": "WAB-1",
                    "fields": {
                        "summary": "Test 1",
                        "status": {
                            "name": "To Do"
                        }
                    }
                },
                {
                    "key": "WAB-2",
                    "fields": {
                        "summary": "Test 2",
                        "status": {
                            "name": "To Do"
                        }
                    }
                }
            ]
        });

        let _m = server
            .mock("POST", "/rest/api/3/search")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response.to_string())
            .create_async()
            .await;

        let client = JiraClient::new(
            server.url(),
            "test@example.com".to_string(),
            "test-token".to_string(),
        );

        let tickets = client.search_with_jql("project = WAB", 5).await.unwrap();
        assert_eq!(tickets.len(), 2);
    }
}
