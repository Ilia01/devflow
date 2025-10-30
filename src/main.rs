use clap::{Parser, Subcommand};
use colored::*;

mod api;
mod config;
mod models;

#[derive(Parser)]
#[command(name = "devflow")]
#[command(version = "0.1.0")]
#[command(about = "Automate your Jira/Git workflow", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        /// (e.g., https://jira.company.com)
        #[arg(short, long)]
        jira_url: Option<String>,
    },

    Start {
        /// (e.g., PROJ-1234)
        ticket_id: String,
    },

    /// Show current ticket and branch status
    Status,

    /// List assigned Jira tickets
    List,

    Commit {
        message: String,
    },

    Done,

    /// Test Jira API connection (temporary)
    #[command(hide = true)]
    TestJira {
        ticket_id: String,
        #[arg(long)]
        url: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        token: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("{}", "DevFlow v0.1.0".bright_cyan().bold());
    println!();

    let result = match cli.command {
        Commands::Init { jira_url: _ } => handle_init().await,

        Commands::Start { ticket_id } => handle_start(&ticket_id).await,

        Commands::Status => handle_status(),

        Commands::List => handle_list().await,

        Commands::Commit { message } => handle_commit(&message),

        Commands::Done => handle_done().await,

        Commands::TestJira {
            ticket_id,
            url,
            email,
            token,
        } => handle_test_jira(&ticket_id, &url, &email, &token).await,
    };

    if let Err(e) = result {
        eprintln!("\n{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    println!();
}

fn handle_commit(message: &str) -> anyhow::Result<()> {
    use colored::*;
    use config::settings::Settings;

    println!("{}", format!("Committing changes...").cyan().bold());
    println!();

    let settings = Settings::load()?;
    let git = api::git::GitClient::new()?;

    let branch = git.current_branch()?;
    let ticket_id = extract_ticket_id(&branch)?;

    let formatted_message = format!(
        "{}\n\n{}: {}/browse/{}",
        message,
        ticket_id,
        settings.jira.url,
        ticket_id
    );

    git.commit(&formatted_message)?;

    println!();
    println!("{}", "Commit created successfully!".green().bold());
    println!("  {} {}", "Message:".bold(), message);
    println!("  {} {}", "Ticket:".bold(), ticket_id.bright_white());

    Ok(())
}

async fn handle_done() -> anyhow::Result<()> {
    use colored::*;
    use config::settings::Settings;

    println!("{}", "Finalizing work...".cyan().bold());
    println!();

    let settings = Settings::load()?;
    let git = api::git::GitClient::new()?;

    let branch = git.current_branch()?;
    let ticket_id = extract_ticket_id(&branch)?;

    println!("{}", "  Pushing branch to remote...".dimmed());
    git.push(&branch)?;

    println!("{}", "  Fetching ticket information...".dimmed());
    let jira = api::jira::JiraClient::new(
        settings.jira.url.clone(),
        settings.jira.email.clone(),
        settings.jira.api_token.clone(),
    );

    let ticket = jira.get_ticket(&ticket_id).await?;

    println!("{}", "  Creating merge request...".dimmed());
    let gitlab = api::gitlab::GitLabClient::new(
        settings.git.base_url.clone(),
        settings.git.token.clone(),
    );

    let project_path = std::env::current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mr_title = format!("{}: {}", ticket_id, ticket.fields.summary);
    let mr_description = format!(
        "Resolves {}\n\nJira: {}/browse/{}",
        ticket_id,
        settings.jira.url,
        ticket_id
    );

    let mr_url = gitlab
        .create_merge_request(&project_path, &branch, "main", &mr_title, &mr_description)
        .await?;

    println!("{}", "  Updating Jira status to 'In Review'...".dimmed());
    match jira.update_status(&ticket_id, "In Review").await {
        Ok(_) => {
            println!("{}", "  ✓ Status updated to 'In Review'".green());
        }
        Err(e) => {
            println!("{}", format!("  Could not update status: {}", e).yellow());
            println!("{}", "    (Continuing anyway...)".dimmed());
        }
    }

    println!();
    println!("{}", "All done! Ready for review!".green().bold());
    println!("  {} {}", "Ticket:".bold(), ticket_id.bright_white());
    println!("  {} {}", "Branch:".bold(), branch.bright_white());
    println!("  {} {}", "MR:".bold(), mr_url.bright_cyan());

    Ok(())
}

async fn handle_start(ticket_id: &str) -> anyhow::Result<()> {
    use colored::*;
    use config::settings::Settings;

    println!(
        "{}",
        format!("Starting work on {}...", ticket_id).cyan().bold()
    );
    println!();

    let settings = Settings::load()?;

    let git = api::git::GitClient::new()?;

    if let Ok(current_branch) = git.current_branch() {
        if current_branch.contains(ticket_id) {
            println!(
                "{}",
                format!("  Already on branch: {}", current_branch).yellow()
            );
            println!("{}", "  Run 'devflow status' to see current state".dimmed());
            return Ok(());
        }
    }

    println!("{}", "  Fetching Jira ticket...".dimmed());
    let jira = api::jira::JiraClient::new(
        settings.jira.url.clone(),
        settings.jira.email.clone(),
        settings.jira.api_token.clone(),
    );

    let ticket = jira.get_ticket(ticket_id).await?;

    println!(
        "{}",
        format!("  ✓ Found: {}", ticket.fields.summary).green()
    );
    println!(
        "{}",
        format!("    Status: {}", ticket.fields.status.name).dimmed()
    );

    let branch_name = format_branch_name(
        &settings.preferences.branch_prefix,
        ticket_id,
        &ticket.fields.summary,
    );

    println!();
    println!("{}", format!("  Creating branch: {}", branch_name).cyan());
    git.create_branch(&branch_name)?;

    println!(
        "{}",
        format!(
            "  Updating Jira status to '{}'...",
            settings.preferences.default_transition
        )
        .cyan()
    );

    match jira
        .update_status(ticket_id, &settings.preferences.default_transition)
        .await
    {
        Ok(_) => {
            println!(
                "{}",
                format!(
                    "  ✓ Status updated to '{}'",
                    settings.preferences.default_transition
                )
                .green()
            );
        }
        Err(e) => {
            println!("{}", format!("  Could not update status: {}", e).yellow());
            println!("{}", "    (Continuing anyway...)".dimmed());
        }
    }

    println!();
    println!("{}", "✨ All set! You're ready to code!".green().bold());
    println!();
    println!("  {} {}", "Ticket:".bold(), ticket_id.bright_white());
    println!("  {} {}", "Branch:".bold(), branch_name.bright_white());
    println!("  {} {}", "Summary:".bold(), ticket.fields.summary.dimmed());

    Ok(())
}

fn extract_ticket_id(branch_name: &str) -> anyhow::Result<String> {
    let parts: Vec<&str> = branch_name.split('/').collect();

    if parts.len() < 2 {
        anyhow::bail!("Branch name doesn't contain a ticket ID");
    }

    let ticket_part = parts[1];

    if ticket_part.contains('-') {
        let ticket_id = ticket_part.split('-')
            .take(2)
            .collect::<Vec<_>>()
            .join("-");

        if ticket_id.is_empty() {
            anyhow::bail!("Could not extract ticket ID from branch name");
        }

        Ok(ticket_id)
    } else {
        anyhow::bail!("Branch name doesn't contain a valid ticket ID format")
    }
}

fn format_branch_name(prefix: &str, ticket_id: &str, summary: &str) -> String {
    let slug = summary
        .to_lowercase()
        .split(|c: char| matches!(c, ' ' | ':' | '!' | '?' | ',' | ';' | '.'))
        .filter_map(|word| {
            let cleaned: String = word
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect();
            if cleaned.len() > 1 {
                Some(cleaned)
            } else {
                None
            }
        })
        .take(5)
        .collect::<Vec<_>>()
        .join("_");

    if slug.is_empty() {
        format!("{}/{}", prefix, ticket_id)
    } else {
        format!("{}/{}/{}", prefix, ticket_id, slug)
    }
}

async fn handle_list() -> anyhow::Result<()> {
    use colored::*;
    use config::settings::Settings;

    println!("{}", "Your Assigned Tickets".cyan().bold());
    println!();

    let settings = Settings::load()?;
    let jira = api::jira::JiraClient::new(
        settings.jira.url.clone(),
        settings.jira.email.clone(),
        settings.jira.api_token.clone(),
    );

    let tickets = jira.search_tickets(&settings.jira.project_key).await?;

    if tickets.is_empty() {
        println!("{}", "  No tickets assigned to you".dimmed());
        return Ok(());
    }

    println!("{}  {} tickets found", "".dimmed(), tickets.len().to_string().bright_white());
    println!();

    for ticket in tickets {
        let status_color = match ticket.fields.status.name.as_str() {
            "In Progress" => ticket.fields.status.name.green(),
            "To Do" => ticket.fields.status.name.yellow(),
            "In Review" | "Code Review" => ticket.fields.status.name.blue(),
            "Done" => ticket.fields.status.name.bright_black(),
            _ => ticket.fields.status.name.normal(),
        };

        println!("  {} [{}]  {}",
            ticket.key.bright_white().bold(),
            status_color,
            ticket.fields.summary
        );
    }

    Ok(())
}

fn handle_status() -> anyhow::Result<()> {
    use colored::*;

    println!("{}", "Current Status".cyan());
    println!();

    match api::git::GitClient::new() {
        Ok(git) => {
            match git.current_branch() {
                Ok(branch) => {
                    println!("  {} {}", "Branch:".bold(), branch.bright_white());
                }
                Err(e) => {
                    println!("  {} {}", "Branch:".bold(), format!("Error: {}", e).red());
                }
            }

            match git.status_summary() {
                Ok(summary) => {
                    println!("\n  {}:", "Status".bold());
                    println!("{}", summary);
                }
                Err(e) => {
                    println!("  {} {}", "Status:".bold(), format!("Error: {}", e).red());
                }
            }
        }
        Err(e) => {
            println!("  {}", "Not in a git repository".yellow());
            println!("  {}", e.to_string().dimmed());
        }
    }

    Ok(())
}

async fn handle_init() -> anyhow::Result<()> {
    use colored::*;
    use config::settings::*;

    println!("{}", "DevFlow Configuration Setup".cyan().bold());
    println!();
    println!(
        "{}",
        "This will store your credentials in ~/.devflow/config.toml".dimmed()
    );
    println!(
        "{}",
        "The file will be created with read-only permissions (600)".dimmed()
    );
    println!();

    println!("{}", "Jira Configuration".bold());
    let jira_url = prompt("Jira URL (e.g., https://jira.<company>.com)")?;
    let jira_email = prompt("Jira email")?;
    println!();
    println!("{}", "To create a Jira API token:".dimmed());
    println!(
        "{}",
        "  1. Go to https://id.atlassian.com/manage-profile/security/api-tokens".dimmed()
    );
    println!("{}", "  2. Click 'Create API token'".dimmed());
    println!("{}", "  3. Copy and paste it here".dimmed());
    println!();
    let jira_token = prompt_password("Jira API token")?;
    let project_key = prompt("Default project key (e.g., WBA)")?;

    println!();
    println!("{}", "=== Git Configuration ===".bold());
    let git_provider = prompt_with_default("Git provider (gitlab/github)", "gitlab")?;
    let git_url = prompt("Git base URL (e.g., https://git.<company>.com)")?;
    println!();
    println!(
        "{}",
        "For GitLab, create a token at: Settings > Access Tokens".dimmed()
    );
    println!(
        "{}",
        "For GitHub, create at: Settings > Developer settings > Personal access tokens".dimmed()
    );
    println!();
    let git_token = prompt_password("Git API token")?;

    println!();
    println!("{}", "=== Preferences ===".bold());
    let branch_prefix = prompt_with_default("Branch prefix (feat/fix/test)", "feat")?;
    let default_transition = prompt_with_default("Default Jira transition", "In Progress")?;

    let settings = Settings {
        jira: JiraConfig {
            url: jira_url,
            email: jira_email,
            api_token: jira_token,
            project_key,
        },
        git: GitConfig {
            provider: git_provider,
            base_url: git_url,
            token: git_token,
        },
        preferences: Preferences {
            branch_prefix,
            default_transition,
        },
    };

    settings.save()?;

    let config_path = Settings::config_dir()?.join("config.toml");
    println!();
    println!("{}", "Configuration saved!".green().bold());
    println!(
        "  Location: {}",
        config_path.display().to_string().bright_white()
    );
    println!();
    println!("{}", "Keep your API tokens secure!".yellow());
    println!("{}", "  Never commit config.toml to git".dimmed());

    Ok(())
}

fn prompt(message: &str) -> anyhow::Result<String> {
    use std::io::Write;
    print!("{}: ", message.bright_white());
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_password(message: &str) -> anyhow::Result<String> {
    use std::io::Write;
    print!("{}: ", message.bright_white());
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn prompt_with_default(message: &str, default: &str) -> anyhow::Result<String> {
    use std::io::Write;
    print!("{} [{}]: ", message.bright_white(), default.dimmed());
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

async fn handle_test_jira(
    ticket_id: &str,
    url: &str,
    email: &str,
    token: &str,
) -> anyhow::Result<()> {
    use colored::*;

    println!("{}", "Testing Jira API connection...".cyan());
    println!();

    let jira = api::jira::JiraClient::new(url.to_string(), email.to_string(), token.to_string());

    println!("{}", format!("  Fetching ticket {}...", ticket_id).dimmed());

    let ticket = jira.get_ticket(ticket_id).await?;

    println!();
    println!("{}", "✓ Successfully fetched ticket!".green().bold());
    println!();
    println!("  {} {}", "Key:".bold(), ticket.key.bright_white());
    println!(
        "  {} {}",
        "Summary:".bold(),
        ticket.fields.summary.bright_white()
    );
    println!(
        "  {} {}",
        "Status:".bold(),
        ticket.fields.status.name.yellow()
    );

    if let Some(assignee) = &ticket.fields.assignee {
        println!(
            "  {} {}",
            "Assignee:".bold(),
            assignee.display_name.bright_white()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_branch_name_basic() {
        let result = format_branch_name("feat", "WAB-1234", "Add user authentication");
        assert_eq!(result, "feat/WAB-1234/add_user_authentication");
    }

    #[test]
    fn test_format_branch_name_with_special_chars() {
        let result = format_branch_name("fix", "PROJ-999", "Fix bug: login doesn't work!");
        assert_eq!(result, "fix/PROJ-999/fix_bug_login_doesnt_work");
    }

    #[test]
    fn test_format_branch_name_long_summary() {
        let result = format_branch_name(
            "feat",
            "WAB-123",
            "This is a very long summary that should be truncated to only five words",
        );
        assert_eq!(result, "feat/WAB-123/this_is_very_long_summary");
    }

    #[test]
    fn test_format_branch_name_with_numbers() {
        let result = format_branch_name("feat", "ABC-42", "Update Node.js to v20");
        assert_eq!(result, "feat/ABC-42/update_node_js_to_v20");
    }

    #[test]
    fn test_format_branch_name_empty_summary() {
        let result = format_branch_name("test", "TICKET-1", "");
        assert_eq!(result, "test/TICKET-1");
    }

    #[test]
    fn test_format_branch_name_real_example() {
        let result = format_branch_name("feat", "WAB-3848", "Implement attempts doc logic");
        assert_eq!(result, "feat/WAB-3848/implement_attempts_doc_logic");
    }

    #[test]
    fn test_extract_ticket_id_basic() {
        let result = extract_ticket_id("feat/WAB-3848/implement_attempts_doc_logic");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "WAB-3848");
    }

    #[test]
    fn test_extract_ticket_id_short_branch() {
        let result = extract_ticket_id("feat/PROJ-123");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "PROJ-123");
    }

    #[test]
    fn test_extract_ticket_id_no_slash() {
        let result = extract_ticket_id("main");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_ticket_id_no_dash() {
        let result = extract_ticket_id("feat/nodash");
        assert!(result.is_err());
    }
}
