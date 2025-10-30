use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct GitLabClient {
    client: Client,
    base_url: String,
    token: String,
}

#[derive(Debug, Serialize)]
struct CreateMergeRequestPayload {
    source_branch: String,
    target_branch: String,
    title: String,
    description: String,
    remove_source_branch: bool,
}

#[derive(Debug, Deserialize)]
struct MergeRequest {
    web_url: String,
}

#[derive(Debug, Deserialize)]
struct Project {
    id: u64,
}

impl GitLabClient {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            token,
        }
    }

    pub async fn create_merge_request(
        &self,
        project_path: &str,
        source_branch: &str,
        target_branch: &str,
        title: &str,
        description: &str,
    ) -> Result<String> {
        let project_id = self.get_project_id(project_path).await?;

        let payload = CreateMergeRequestPayload {
            source_branch: source_branch.to_string(),
            target_branch: target_branch.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            remove_source_branch: true,
        };

        let url = format!(
            "{}/api/v4/projects/{}/merge_requests",
            self.base_url, project_id
        );

        let response = self
            .client
            .post(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .json(&payload)
            .send()
            .await
            .context("Failed to send merge request creation request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("GitLab API error ({}): {}", status, text);
        }

        let mr = response
            .json::<MergeRequest>()
            .await
            .context("Failed to parse merge request response")?;

        Ok(mr.web_url)
    }

    async fn get_project_id(&self, project_path: &str) -> Result<u64> {
        let encoded_path = urlencoding::encode(project_path);
        let url = format!("{}/api/v4/projects/{}", self.base_url, encoded_path);

        let response = self
            .client
            .get(&url)
            .header("PRIVATE-TOKEN", &self.token)
            .send()
            .await
            .context("Failed to fetch project information")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("GitLab API error ({}): {}", status, text);
        }

        let project = response
            .json::<Project>()
            .await
            .context("Failed to parse project response")?;

        Ok(project.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gitlab_client_creation() {
        let client = GitLabClient::new(
            "https://git.example.com".to_string(),
            "test-token".to_string(),
        );
        assert_eq!(client.base_url, "https://git.example.com");
        assert_eq!(client.token, "test-token");
    }
}
