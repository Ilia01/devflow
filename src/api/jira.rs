use crate::config::settings::AuthMethod;
use crate::models::ticket::JiraTicket;
use anyhow::{Context, Result};
use reqwest::{Client, RequestBuilder};

enum AuthConfig {
    BearerToken { token: String },
    BasicAuth { email: String, api_token: String },
}

pub struct JiraClient {
    client: Client,
    base_url: String,
    auth: AuthConfig,
}

impl JiraClient {
    pub fn new(base_url: String, email: String, auth_method: AuthMethod) -> Self {
        let auth = match auth_method {
            AuthMethod::PersonalAccessToken { token } => AuthConfig::BearerToken { token },
            AuthMethod::ApiToken { token } => AuthConfig::BasicAuth {
                email: email.clone(),
                api_token: token
            },
        };

        Self {
            client: Client::new(),
            base_url,
            auth,
        }
    }

    fn apply_auth(&self, builder: RequestBuilder) -> RequestBuilder {
        match &self.auth {
            AuthConfig::BearerToken { token } => {
                builder.header("Authorization", format!("Bearer {}", token))
            }
            AuthConfig::BasicAuth { email, api_token } => {
                builder.basic_auth(email, Some(api_token))
            }
        }
    }

    pub async fn get_ticket(&self, ticket_id: &str) -> Result<JiraTicket> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, ticket_id);

        let response = self.apply_auth(self.client.get(&url))
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

        let transitions_response = self.apply_auth(self.client.get(&transitions_url))
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

        let response = self.apply_auth(self.client.post(&transitions_url))
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
        self.search_with_jql(&jql, 50).await
    }

    pub async fn search_with_jql(&self, jql: &str, max_results: u32) -> Result<Vec<crate::models::ticket::JiraTicket>> {
        let url = format!("{}/rest/api/3/search", self.base_url);

        let body = serde_json::json!({
            "jql": jql,
            "fields": ["summary", "status", "assignee"],
            "maxResults": max_results
        });

        let response = self.apply_auth(self.client.post(&url))
            .json(&body)
            .send()
            .await
            .context("Failed to send search request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Jira search API error ({}): {}", status, text);
        }

        let result: serde_json::Value = response.json().await.context("Failed to parse search response as JSON")?;

        // Debug: Print raw response if verbose mode or if parsing fails
        if std::env::var("DEVFLOW_DEBUG").is_ok() {
            eprintln!("DEBUG: Raw Jira response:\n{}", serde_json::to_string_pretty(&result).unwrap_or_default());
        }

        let issues = result["issues"].as_array().context("No 'issues' field in response")?;

        let mut tickets: Vec<crate::models::ticket::JiraTicket> = Vec::new();
        let mut parse_errors: Vec<String> = Vec::new();

        for (idx, issue) in issues.iter().enumerate() {
            match serde_json::from_value::<crate::models::ticket::JiraTicket>(issue.clone()) {
                Ok(ticket) => tickets.push(ticket),
                Err(e) => {
                    parse_errors.push(format!("Issue {}: {}", idx, e));
                    if std::env::var("DEVFLOW_DEBUG").is_ok() {
                        eprintln!("DEBUG: Failed to parse issue {}:\n{}", idx, serde_json::to_string_pretty(issue).unwrap_or_default());
                    }
                }
            }
        }

        // If we have parse errors and debug is on, or if ALL tickets failed to parse, report it
        if !parse_errors.is_empty() {
            if tickets.is_empty() {
                anyhow::bail!(
                    "Failed to parse any tickets from response. Errors:\n{}\n\nRun with DEVFLOW_DEBUG=1 to see raw response",
                    parse_errors.join("\n")
                );
            } else if std::env::var("DEVFLOW_DEBUG").is_ok() {
                eprintln!("WARNING: Some tickets failed to parse: {}", parse_errors.join(", "));
            }
        }

        Ok(tickets)
    }

    /// Test connection without parsing tickets - just validates auth and API access
    pub async fn test_connection(&self) -> Result<()> {
        let url = format!("{}/rest/api/3/myself", self.base_url);

        let response = self.apply_auth(self.client.get(&url))
            .send()
            .await
            .context("Failed to connect to Jira")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Jira API error ({}): {}", status, text);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jira_client_creation_with_api_token() {
        let client = JiraClient::new(
            "https://jira.example.com".to_string(),
            "test@example.com".to_string(),
            AuthMethod::ApiToken {
                token: "test-token".to_string(),
            },
        );
        assert_eq!(client.base_url, "https://jira.example.com");
        assert!(matches!(client.auth, AuthConfig::BasicAuth { .. }));
    }

    #[test]
    fn test_jira_client_creation_with_pat() {
        let client = JiraClient::new(
            "https://jira.example.com".to_string(),
            "test@example.com".to_string(),
            AuthMethod::PersonalAccessToken {
                token: "pat-token".to_string(),
            },
        );
        assert_eq!(client.base_url, "https://jira.example.com");
        assert!(matches!(client.auth, AuthConfig::BearerToken { .. }));
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
            AuthMethod::ApiToken {
                token: "test-token".to_string(),
            },
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
            AuthMethod::ApiToken {
                token: "test-token".to_string(),
            },
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
            AuthMethod::ApiToken {
                token: "invalid-token".to_string(),
            },
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
            AuthMethod::ApiToken {
                token: "test-token".to_string(),
            },
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
            AuthMethod::ApiToken {
                token: "test-token".to_string(),
            },
        );

        let result = client.search_tickets("WAB").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No 'issues' field in response"));
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
            AuthMethod::ApiToken {
                token: "test-token".to_string(),
            },
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
            AuthMethod::ApiToken {
                token: "test-token".to_string(),
            },
        );

        let tickets = client.search_with_jql("project = WAB", 5).await.unwrap();
        assert_eq!(tickets.len(), 2);
    }
}
