use colored::*;
use std::fmt;

#[derive(Debug)]
pub enum DevFlowError {
    // Configuration errors
    ConfigNotFound,
    ConfigInvalid(String),
    ConfigValidationFailed(String),

    // Jira errors
    JiraAuthFailed(u16),
    JiraTicketNotFound(String),
    JiraApiError(u16, String),
    JiraTransitionNotFound(String),

    // Git errors
    NotInGitRepo,
    GitRepoNotClean,
    BranchAlreadyExists(String),
    BranchHasNoTicketId(String),
    NoPushAccess(String),

    // GitHub/GitLab errors
    PrCreationFailed(String),
    GitHubAuthFailed,
    GitLabAuthFailed,

    // Network errors
    NetworkError(String),

    // Generic error
    Other(String),
}

impl fmt::Display for DevFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Configuration errors
            DevFlowError::ConfigNotFound => {
                write!(f, "{}\n", "Configuration not found".red().bold())?;
                write!(f, "   {}\n\n", "Run 'devflow init' to set up your configuration".dimmed())?;
                write!(f, "   {}", "devflow init".green())
            }
            DevFlowError::ConfigInvalid(msg) => {
                write!(f, "{}\n", "Invalid configuration".red().bold())?;
                write!(f, "   {}\n\n", msg.dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Check your config file: ~/.devflow/config.toml\n")?;
                write!(f, "   2. Or reinitialize: {}", "devflow init".green())
            }
            DevFlowError::ConfigValidationFailed(msg) => {
                write!(f, "{}\n", "Configuration validation failed".red().bold())?;
                write!(f, "   {}\n\n", msg.dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Check your API tokens are valid\n")?;
                write!(f, "   2. Verify network connectivity\n")?;
                write!(f, "   3. Reinitialize if needed: {}", "devflow init".green())
            }

            // Jira errors
            DevFlowError::JiraAuthFailed(status) => {
                write!(f, "{}\n", format!("Jira authentication failed ({})", status).red().bold())?;
                write!(f, "   {}\n\n", "Your API token may have expired or is invalid".dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Generate new token: {}\n", "https://id.atlassian.com/manage-profile/security/api-tokens".cyan())?;
                write!(f, "   2. Update config: {}\n", "devflow init".green())?;
                write!(f, "   3. Or edit manually: ~/.devflow/config.toml")
            }
            DevFlowError::JiraTicketNotFound(ticket_id) => {
                write!(f, "{}\n", format!("Ticket '{}' not found", ticket_id).red().bold())?;
                write!(f, "   {}\n\n", "The ticket doesn't exist or you don't have access to it".dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Check the ticket ID is correct\n")?;
                write!(f, "   2. Verify you have access to this project\n")?;
                write!(f, "   3. Search for tickets: {}", format!("devflow search \"{}\"", ticket_id).green())
            }
            DevFlowError::JiraApiError(status, msg) => {
                write!(f, "{}\n", format!("Jira API error ({})", status).red().bold())?;
                write!(f, "   {}\n\n", msg.dimmed())?;
                write!(f, "   Try again or check your network connection")
            }
            DevFlowError::JiraTransitionNotFound(transition) => {
                write!(f, "{}\n", format!("Status transition '{}' not found", transition).red().bold())?;
                write!(f, "   {}\n\n", "This status is not available for this ticket".dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Check available statuses in Jira\n")?;
                write!(f, "   2. Update your config with a valid transition")
            }

            // Git errors
            DevFlowError::NotInGitRepo => {
                write!(f, "{}\n", "Not in a git repository".red().bold())?;
                write!(f, "   {}\n\n", "DevFlow must be run inside a git repository".dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Navigate to a git repository\n")?;
                write!(f, "   2. Or initialize one: {}", "git init".green())
            }
            DevFlowError::GitRepoNotClean => {
                write!(f, "{}\n", "Uncommitted changes detected".red().bold())?;
                write!(f, "   {}\n\n", "Commit or stash your changes before running 'devflow done'".dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Commit changes: {}\n", "devflow commit \"your message\"".green())?;
                write!(f, "   2. Or stash: {}\n", "git stash".green())?;
                write!(f, "   3. Check status: {}", "git status".green())
            }
            DevFlowError::BranchAlreadyExists(branch) => {
                write!(f, "{}\n", format!("Branch '{}' already exists", branch).red().bold())?;
                write!(f, "   {}\n\n", "You're already on this branch or it exists locally".dimmed())?;
                write!(f, "   To check status: {}", "devflow status".green())
            }
            DevFlowError::BranchHasNoTicketId(branch) => {
                write!(f, "{}\n", "Branch doesn't contain a ticket ID".red().bold())?;
                write!(f, "   {}\n\n", format!("Current branch: {}", branch).dimmed())?;
                write!(f, "   DevFlow expects branches in format: feat/TICKET-123/description\n\n")?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Start work on a ticket: {}\n", "devflow start TICKET-123".green())?;
                write!(f, "   2. Or switch to a DevFlow branch")
            }
            DevFlowError::NoPushAccess(msg) => {
                write!(f, "{}\n", "Failed to push to remote".red().bold())?;
                write!(f, "   {}\n\n", msg.dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Check your SSH keys are configured\n")?;
                write!(f, "   2. Verify you have push access to the repository\n")?;
                write!(f, "   3. Test SSH: {}", "ssh -T git@github.com".green())
            }

            // GitHub/GitLab errors
            DevFlowError::PrCreationFailed(msg) => {
                write!(f, "{}\n", "Failed to create pull/merge request".red().bold())?;
                write!(f, "   {}\n\n", msg.dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Check your API token is valid\n")?;
                write!(f, "   2. Verify you have permissions to create PRs\n")?;
                write!(f, "   3. Try creating the PR manually")
            }
            DevFlowError::GitHubAuthFailed => {
                write!(f, "{}\n", "GitHub authentication failed".red().bold())?;
                write!(f, "   {}\n\n", "Your GitHub token is invalid or expired".dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Generate new token: Settings > Developer settings > Personal access tokens\n")?;
                write!(f, "   2. Required scope: repo (full control)\n")?;
                write!(f, "   3. Update config: {}", "devflow init".green())
            }
            DevFlowError::GitLabAuthFailed => {
                write!(f, "{}\n", "GitLab authentication failed".red().bold())?;
                write!(f, "   {}\n\n", "Your GitLab token is invalid or expired".dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Generate new token: Settings > Access Tokens\n")?;
                write!(f, "   2. Required scope: api\n")?;
                write!(f, "   3. Update config: {}", "devflow init".green())
            }

            // Network errors
            DevFlowError::NetworkError(msg) => {
                write!(f, "{}\n", "Network error".red().bold())?;
                write!(f, "   {}\n\n", msg.dimmed())?;
                write!(f, "   To fix:\n")?;
                write!(f, "   1. Check your internet connection\n")?;
                write!(f, "   2. Verify you can reach the API endpoints\n")?;
                write!(f, "   3. Try again in a moment")
            }

            // Generic
            DevFlowError::Other(msg) => {
                write!(f, "{}\n", "Error".red().bold())?;
                write!(f, "   {}", msg.dimmed())
            }
        }
    }
}

impl std::error::Error for DevFlowError {}

// Conversion from anyhow::Error
impl From<anyhow::Error> for DevFlowError {
    fn from(err: anyhow::Error) -> Self {
        DevFlowError::Other(err.to_string())
    }
}

// Helper to convert common error types
impl From<std::io::Error> for DevFlowError {
    fn from(err: std::io::Error) -> Self {
        DevFlowError::Other(err.to_string())
    }
}

impl From<reqwest::Error> for DevFlowError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() || err.is_connect() {
            DevFlowError::NetworkError(err.to_string())
        } else if let Some(status) = err.status() {
            if status == 401 || status == 403 {
                DevFlowError::JiraAuthFailed(status.as_u16())
            } else {
                DevFlowError::Other(err.to_string())
            }
        } else {
            DevFlowError::Other(err.to_string())
        }
    }
}

pub type Result<T> = std::result::Result<T, DevFlowError>;
