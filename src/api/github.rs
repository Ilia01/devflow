use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct GitHubClient {
    client: Client,
    owner: String,
    repo: String,
    token: String,
}

#[derive(Debug, Serialize)]
struct CreatePullRequestPayload {
    title: String,
    body: String,
    head: String,
    base: String,
}

#[derive(Debug, Deserialize)]
struct PullRequest {
    html_url: String,
    #[allow(dead_code)]
    number: u64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Repository {
    full_name: String,
}

impl GitHubClient {
    pub fn new(owner: String, repo: String, token: String) -> Self {
        Self {
            client: Client::new(),
            owner,
            repo,
            token,
        }
    }

    pub async fn create_pull_request(
        &self,
        source_branch: &str,
        target_branch: &str,
        title: &str,
        description: &str,
    ) -> Result<String> {
        let payload = CreatePullRequestPayload {
            title: title.to_string(),
            body: description.to_string(),
            head: source_branch.to_string(),
            base: target_branch.to_string(),
        };

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls",
            self.owner, self.repo
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "devflow-cli")
            .json(&payload)
            .send()
            .await
            .context("Failed to send pull request creation request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, text);
        }

        let pr = response
            .json::<PullRequest>()
            .await
            .context("Failed to parse pull request response")?;

        Ok(pr.html_url)
    }

    pub async fn get_repo_info(&self) -> Result<String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}",
            self.owner, self.repo
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "devflow-cli")
            .send()
            .await
            .context("Failed to fetch repository information")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error ({}): {}", status, text);
        }

        let repo = response
            .json::<Repository>()
            .await
            .context("Failed to parse repository response")?;

        Ok(repo.full_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_client_creation() {
        let client = GitHubClient::new(
            "owner".to_string(),
            "repo".to_string(),
            "test-token".to_string(),
        );
        assert_eq!(client.owner, "owner");
        assert_eq!(client.repo, "repo");
        assert_eq!(client.token, "test-token");
    }
}
