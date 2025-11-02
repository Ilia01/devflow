use clap::{Parser, Subcommand};
use colored::*;

mod api;
mod config;
mod errors;
mod models;

#[derive(Parser)]
#[command(name = "devflow")]
#[command(version = "0.1.0")]
#[command(about = "Automate your Jira/Git workflow", long_about = None)]
struct Cli {
    /// for debugging purposes
    #[arg(short, long, global = true)]
    verbose: bool,

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
    List {
        /// Filter by status (e.g., "To Do", "In Progress")
        #[arg(long)]
        status: Option<String>,

        /// Filter by project key
        #[arg(long)]
        project: Option<String>,

        /// Output as JSON for scripting
        #[arg(long)]
        json: bool,
    },

    /// Search Jira tickets
    Search {
        /// Search text (searches in summary and description)
        query: String,

        /// Filter by assignee (use "me" for current user)
        #[arg(long)]
        assignee: Option<String>,

        /// Filter by status (e.g., "To Do", "In Progress")
        #[arg(long)]
        status: Option<String>,

        /// Filter by project key
        #[arg(long)]
        project: Option<String>,

        /// Maximum number of results (default: 10)
        #[arg(long, default_value = "10")]
        limit: u32,

        /// Interactive mode - select a ticket to start working on
        #[arg(long, short)]
        interactive: bool,
    },

    /// Open ticket or PR in browser
    Open {
        /// Optional ticket ID (e.g., WAB-1234). If not provided, uses current branch
        ticket_id: Option<String>,

        /// Open the PR/MR instead of the ticket
        #[arg(long)]
        pr: bool,

        /// Open the Jira board instead of ticket
        #[arg(long)]
        board: bool,
    },

    Commit {
        message: String,
    },

    Done,

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

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

#[derive(Subcommand)]
enum ConfigAction {
    /// Display current configuration (with masked secrets)
    Show,

    /// Set a specific configuration value
    Set {
        /// Configuration key (e.g., jira.email, jira.url, git.token)
        key: String,
        /// New value
        value: String,
    },

    /// Validate configuration by testing API connections
    Validate,

    /// Get the path to the config file
    Path,
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

        Commands::List { status, project, json } => {
            handle_list(status.as_deref(), project.as_deref(), json).await
        }

        Commands::Search { query, assignee, status, project, limit, interactive } => {
            handle_search(&query, assignee.as_deref(), status.as_deref(), project.as_deref(), limit, interactive).await
        }

        Commands::Open { ticket_id, pr, board } => handle_open(ticket_id.as_deref(), pr, board).await,

        Commands::Commit { message } => handle_commit(&message),

        Commands::Done => handle_done().await,

        Commands::Config { action } => handle_config(action).await,

        Commands::TestJira {
            ticket_id,
            url,
            email,
            token,
        } => handle_test_jira(&ticket_id, &url, &email, &token).await,
    };

    if let Err(e) = result {
        eprintln!("\n{}", e);
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

    let settings = Settings::load().map_err(|e| anyhow::anyhow!("{}", e))?;
    let git = api::git::GitClient::new().map_err(|e| anyhow::anyhow!("{}", e))?;

    // Check if working directory is clean
    if !git.is_clean().map_err(|e| anyhow::anyhow!("{}", e))? {
        return Err(anyhow::anyhow!("{}", errors::DevFlowError::GitRepoNotClean));
    }

    let branch = git.current_branch().map_err(|e| anyhow::anyhow!("{}", e))?;
    let ticket_id = extract_ticket_id(&branch)?;

    println!("{}", "  Pushing branch to remote...".dimmed());
    git.push(&branch)?;

    println!("{}", "  Fetching ticket information...".dimmed());
    let jira = api::jira::JiraClient::new(
        settings.jira.url.clone(),
        settings.jira.email.clone(),
        settings.jira.auth_method.clone(),
    );

    let ticket = jira.get_ticket(&ticket_id).await?;

    let pr_title = format!("{}: {}", ticket_id, ticket.fields.summary);
    let pr_description = format!(
        "Resolves {}\n\nJira: {}/browse/{}",
        ticket_id,
        settings.jira.url,
        ticket_id
    );

    let pr_url = if settings.git.provider.to_lowercase() == "github" {
        println!("{}", "  Creating pull request...".dimmed());
        let owner = settings.git.owner.as_ref()
            .ok_or_else(|| anyhow::anyhow!("GitHub owner not configured"))?;
        let repo = settings.git.repo.as_ref()
            .ok_or_else(|| anyhow::anyhow!("GitHub repo not configured"))?;

        let github = api::github::GitHubClient::new(
            owner.clone(),
            repo.clone(),
            settings.git.token.clone(),
        );

        github
            .create_pull_request(&branch, "main", &pr_title, &pr_description)
            .await?
    } else {
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

        gitlab
            .create_merge_request(&project_path, &branch, "main", &pr_title, &pr_description)
            .await?
    };

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

    let pr_label = if settings.git.provider.to_lowercase() == "github" {
        "PR:"
    } else {
        "MR:"
    };

    println!();
    println!("{}", "All done! Ready for review!".green().bold());
    println!("  {} {}", "Ticket:".bold(), ticket_id.bright_white());
    println!("  {} {}", "Branch:".bold(), branch.bright_white());
    println!("  {} {}", pr_label.bold(), pr_url.bright_cyan());

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
        settings.jira.auth_method.clone(),
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
        return Err(anyhow::anyhow!("{}", errors::DevFlowError::BranchHasNoTicketId(branch_name.to_string())));
    }

    let ticket_part = parts[1];

    if ticket_part.contains('-') {
        let ticket_id = ticket_part.split('-')
            .take(2)
            .collect::<Vec<_>>()
            .join("-");

        if ticket_id.is_empty() {
            return Err(anyhow::anyhow!("{}", errors::DevFlowError::BranchHasNoTicketId(branch_name.to_string())));
        }

        Ok(ticket_id)
    } else {
        Err(anyhow::anyhow!("{}", errors::DevFlowError::BranchHasNoTicketId(branch_name.to_string())))
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

async fn handle_list(
    status_filter: Option<&str>,
    project_filter: Option<&str>,
    json_output: bool,
) -> anyhow::Result<()> {
    use colored::*;
    use config::settings::Settings;

    let settings = Settings::load().map_err(|e| anyhow::anyhow!("{}", e))?;
    let jira = api::jira::JiraClient::new(
        settings.jira.url.clone(),
        settings.jira.email.clone(),
        settings.jira.auth_method.clone(),
    );

    // Build JQL query with filters
    let mut jql_parts = vec!["assignee = currentUser()".to_string()];

    let project_key = project_filter.unwrap_or(&settings.jira.project_key);
    jql_parts.push(format!("project = {}", project_key));

    if let Some(status) = status_filter {
        jql_parts.push(format!("status = \"{}\"", status));
    }

    let jql = jql_parts.join(" AND ");
    let tickets = jira.search_with_jql(&jql, 50).await?;

    // JSON output
    if json_output {
        let json = serde_json::to_string_pretty(&tickets)?;
        println!("{}", json);
        return Ok(());
    }

    // Pretty terminal output
    if !json_output {
        println!("{}", "Your Assigned Tickets".cyan().bold());
        println!();
    }

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

async fn handle_search(
    query: &str,
    assignee: Option<&str>,
    status: Option<&str>,
    project: Option<&str>,
    limit: u32,
    interactive: bool,
) -> anyhow::Result<()> {
    use colored::*;
    use config::settings::Settings;

    println!("{}", format!("Searching for: \"{}\"", query).cyan().bold());
    println!();

    let settings = Settings::load().map_err(|e| anyhow::anyhow!("{}", e))?;
    let jira = api::jira::JiraClient::new(
        settings.jira.url.clone(),
        settings.jira.email.clone(),
        settings.jira.auth_method.clone(),
    );

    let mut jql_parts = Vec::new();

    jql_parts.push(format!("(summary ~ \"{}\" OR description ~ \"{}\")", query, query));

    let project_key = project.unwrap_or(&settings.jira.project_key);
    jql_parts.push(format!("project = {}", project_key));

    if let Some(assignee_val) = assignee {
        if assignee_val == "me" {
            jql_parts.push("assignee = currentUser()".to_string());
        } else {
            jql_parts.push(format!("assignee = \"{}\"", assignee_val));
        }
    }

    if let Some(status_val) = status {
        jql_parts.push(format!("status = \"{}\"", status_val));
    }

    let jql = jql_parts.join(" AND ");

    println!("{}", format!("  JQL: {}", jql).dimmed());
    println!();

    let tickets = jira.search_with_jql(&jql, limit).await?;

    if tickets.is_empty() {
        println!("{}", "  No tickets found".dimmed());
        return Ok(());
    }

    println!("{} {} results", "".dimmed(), tickets.len().to_string().bright_white());
    println!();

    for (i, ticket) in tickets.iter().enumerate() {
        let status_color = match ticket.fields.status.name.as_str() {
            "In Progress" => ticket.fields.status.name.green(),
            "To Do" => ticket.fields.status.name.yellow(),
            "In Review" | "Code Review" => ticket.fields.status.name.blue(),
            "Done" => ticket.fields.status.name.bright_black(),
            _ => ticket.fields.status.name.normal(),
        };

        println!("  {}. {} [{}]  {}",
            (i + 1).to_string().dimmed(),
            ticket.key.bright_white().bold(),
            status_color,
            ticket.fields.summary
        );
    }

    if tickets.len() == limit as usize {
        println!();
        println!("{}", format!("  Showing {} of potentially more results. Use --limit to see more.", limit).dimmed());
    }

    // Interactive mode - let user select a ticket to start work
    if interactive {
        use dialoguer::Select;

        println!();
        let items: Vec<String> = tickets.iter().map(|t| {
            format!("{} [{}] {}", t.key, t.fields.status.name, t.fields.summary)
        }).collect();

        let selection = Select::new()
            .with_prompt("Select a ticket to start working on")
            .items(&items)
            .interact_opt()?;

        if let Some(index) = selection {
            let selected_ticket = &tickets[index];
            println!();
            println!("{}", format!("Starting work on {}...", selected_ticket.key).cyan().bold());

            // Call handle_start with the selected ticket
            return handle_start(&selected_ticket.key).await;
        } else {
            println!("\n{}", "No ticket selected".yellow());
        }
    }

    Ok(())
}

async fn handle_open(ticket_id: Option<&str>, open_pr: bool, open_board: bool) -> anyhow::Result<()> {
    use colored::*;
    use config::settings::Settings;

    let settings = Settings::load()?;

    if open_board {
        let board_url = format!("{}/jira/software/projects/{}/boards",
            settings.jira.url,
            settings.jira.project_key
        );
        println!("{} {}", "Opening board:".dimmed(), board_url.bright_white());
        open::that(&board_url)?;
        return Ok(());
    }

    let ticket_id = if let Some(id) = ticket_id {
        id.to_string()
    } else {
        let git = api::git::GitClient::new()?;
        let branch = git.current_branch()?;
        extract_ticket_id(&branch)?
    };

    if open_pr {
        let git = api::git::GitClient::new()?;
        let branch = git.current_branch()?;

        let pr_url = match settings.git.provider.as_str() {
            "github" => {
                let owner = settings.git.owner.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("GitHub owner not configured"))?;
                let repo = settings.git.repo.as_ref()
                    .ok_or_else(|| anyhow::anyhow!("GitHub repo not configured"))?;
                format!("{}/{}/{}/pulls?q=is%3Apr+head%3A{}",
                    settings.git.base_url.replace("api.", ""),
                    owner,
                    repo,
                    urlencoding::encode(&branch)
                )
            },
            "gitlab" => {
                format!("{}/merge_requests?scope=all&state=opened&source_branch={}",
                    settings.git.base_url,
                    urlencoding::encode(&branch)
                )
            },
            provider => anyhow::bail!("Unsupported provider: {}", provider)
        };

        println!("{} {}", "Opening PR/MR:".dimmed(), pr_url.bright_white());
        open::that(&pr_url)?;
        return Ok(());
    }

    // Default: Open Jira ticket
    let ticket_url = format!("{}/browse/{}", settings.jira.url, ticket_id);
    println!("{} {}", "Opening ticket:".dimmed(), ticket_url.bright_white());
    open::that(&ticket_url)?;

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

    println!("{}", "Select authentication method:".bold());
    println!("{}", "  1. Personal Access Token (for Jira Data Center/Server)".dimmed());
    println!("{}", "  2. API Token (for Jira Cloud)".dimmed());
    let auth_choice = prompt_with_default("Choice (1/2)", "2")?;

    let auth_method = if auth_choice == "1" {
        println!();
        println!("{}", "To create a Personal Access Token:".dimmed());
        println!("{}", "  1. Go to Jira → Profile → Personal Access Tokens".dimmed());
        println!("{}", "  2. Click 'Create token'".dimmed());
        println!("{}", "  3. Copy and paste it here".dimmed());
        println!();
        let token = prompt_password("Personal Access Token")?;
        AuthMethod::PersonalAccessToken { token }
    } else {
        println!();
        println!("{}", "To create a Jira API token:".dimmed());
        println!("{}", "  1. Go to https://id.atlassian.com/manage-profile/security/api-tokens".dimmed());
        println!("{}", "  2. Click 'Create API token'".dimmed());
        println!("{}", "  3. Copy and paste it here".dimmed());
        println!();
        let token = prompt_password("Jira API token")?;
        AuthMethod::ApiToken { token }
    };

    let project_key = prompt("Default project key (e.g., WBA)")?;

    println!();
    println!("{}", "=== Git Configuration ===".bold());
    let git_provider = prompt_with_default("Git provider (gitlab/github)", "gitlab")?;

    let (git_url, git_owner, git_repo) = if git_provider.to_lowercase() == "github" {
        println!();
        println!("{}", "For GitHub, create a token at:".dimmed());
        println!("{}", "  Settings > Developer settings > Personal access tokens > Generate new token".dimmed());
        println!("{}", "  Required scopes: repo (full control)".dimmed());
        println!();
        let owner = prompt("Repository owner (username or org)")?;
        let repo = prompt("Repository name")?;
        ("https://api.github.com".to_string(), Some(owner), Some(repo))
    } else {
        let url = prompt("GitLab base URL (e.g., https://git.<company>.com)")?;
        println!();
        println!("{}", "For GitLab, create a token at:".dimmed());
        println!("{}", "  Settings > Access Tokens".dimmed());
        println!("{}", "  Required scopes: api".dimmed());
        (url, None, None)
    };

    println!();
    let git_token = prompt_password("Git API token")?;

    println!();
    println!("{}", "=== Preferences ===".bold());
    let branch_prefix = prompt_with_default("Branch prefix (feat/fix/test)", "feat")?;
    let default_transition = prompt_with_default("Default Jira transition", "In Progress")?;

    let settings = Settings {
        jira: JiraConfig {
            url: jira_url.clone(),
            email: jira_email.clone(),
            auth_method: auth_method.clone(),
            project_key: project_key.clone(),
        },
        git: GitConfig {
            provider: git_provider.clone(),
            base_url: git_url.clone(),
            token: git_token.clone(),
            owner: git_owner.clone(),
            repo: git_repo.clone(),
        },
        preferences: Preferences {
            branch_prefix,
            default_transition,
        },
    };

    println!();
    println!("{}", "Validating configuration...".cyan());
    println!();

    print!("{}", "  Testing Jira connection... ".dimmed());
    let jira_client = api::jira::JiraClient::new(
        jira_url.clone(),
        jira_email.clone(),
        auth_method.clone(),
    );

    match jira_client.search_with_jql(&format!("project = {}", project_key), 1).await {
        Ok(_) => {
            println!("{}", "✓".green().bold());
        }
        Err(e) => {
            println!("{}", "✗".red().bold());
            return Err(anyhow::anyhow!("{}",
                errors::DevFlowError::ConfigValidationFailed(
                    format!("Jira connection failed: {}", e)
                )
            ));
        }
    }

    print!("{}", "  Checking Git token... ".dimmed());
    if git_token.is_empty() {
        println!("{}", "✗".red().bold());
        return Err(anyhow::anyhow!("{}",
            errors::DevFlowError::ConfigValidationFailed(
                "Git token cannot be empty".to_string()
            )
        ));
    }
    println!("{}", "✓".green().bold());

    println!();
    println!("{}", "✓ All validations passed!".green().bold());
    println!();

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

async fn handle_config(action: ConfigAction) -> anyhow::Result<()> {
    use colored::*;
    use config::settings::Settings;

    match action {
        ConfigAction::Show => {
            let settings = Settings::load()?;

            println!("{}", "Current Configuration".cyan().bold());
            println!();

            println!("{}", "[jira]".bold());
            println!("  {} {}", "url:".dimmed(), settings.jira.url.bright_white());
            println!("  {} {}", "email:".dimmed(), settings.jira.email.bright_white());

            // Mask the token
            let masked_token = match &settings.jira.auth_method {
                config::settings::AuthMethod::PersonalAccessToken { token } => {
                    format!("{}***{}", &token[..4.min(token.len())], &token[token.len().saturating_sub(4)..])
                }
                config::settings::AuthMethod::ApiToken { token } => {
                    format!("{}***{}", &token[..4.min(token.len())], &token[token.len().saturating_sub(4)..])
                }
            };

            let auth_type = match settings.jira.auth_method {
                config::settings::AuthMethod::PersonalAccessToken { .. } => "Personal Access Token",
                config::settings::AuthMethod::ApiToken { .. } => "API Token",
            };

            println!("  {} {}", "auth_method:".dimmed(), auth_type.bright_white());
            println!("  {} {}", "token:".dimmed(), masked_token.yellow());
            println!("  {} {}", "project_key:".dimmed(), settings.jira.project_key.bright_white());

            println!();
            println!("{}", "[git]".bold());
            println!("  {} {}", "provider:".dimmed(), settings.git.provider.bright_white());
            println!("  {} {}", "base_url:".dimmed(), settings.git.base_url.bright_white());

            let masked_git_token = format!(
                "{}***{}",
                &settings.git.token[..4.min(settings.git.token.len())],
                &settings.git.token[settings.git.token.len().saturating_sub(4)..]
            );
            println!("  {} {}", "token:".dimmed(), masked_git_token.yellow());

            if let Some(owner) = &settings.git.owner {
                println!("  {} {}", "owner:".dimmed(), owner.bright_white());
            }
            if let Some(repo) = &settings.git.repo {
                println!("  {} {}", "repo:".dimmed(), repo.bright_white());
            }

            println!();
            println!("{}", "[preferences]".bold());
            println!("  {} {}", "branch_prefix:".dimmed(), settings.preferences.branch_prefix.bright_white());
            println!("  {} {}", "default_transition:".dimmed(), settings.preferences.default_transition.bright_white());

            Ok(())
        }

        ConfigAction::Set { key, value } => {
            let mut settings = Settings::load()?;

            // Parse the key to determine what to set
            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() != 2 {
                return Err(anyhow::anyhow!("Invalid key format. Use format: section.field (e.g., jira.email)"));
            }

            let section = parts[0];
            let field = parts[1];

            match (section, field) {
                ("jira", "url") => settings.jira.url = value.clone(),
                ("jira", "email") => settings.jira.email = value.clone(),
                ("jira", "token") => {
                    // Update the token in the existing auth method
                    settings.jira.auth_method = match settings.jira.auth_method {
                        config::settings::AuthMethod::PersonalAccessToken { .. } => {
                            config::settings::AuthMethod::PersonalAccessToken { token: value.clone() }
                        }
                        config::settings::AuthMethod::ApiToken { .. } => {
                            config::settings::AuthMethod::ApiToken { token: value.clone() }
                        }
                    };
                }
                ("jira", "project_key") => settings.jira.project_key = value.clone(),
                ("git", "provider") => settings.git.provider = value.clone(),
                ("git", "base_url") => settings.git.base_url = value.clone(),
                ("git", "token") => settings.git.token = value.clone(),
                ("git", "owner") => settings.git.owner = Some(value.clone()),
                ("git", "repo") => settings.git.repo = Some(value.clone()),
                ("preferences", "branch_prefix") => settings.preferences.branch_prefix = value.clone(),
                ("preferences", "default_transition") => settings.preferences.default_transition = value.clone(),
                _ => return Err(anyhow::anyhow!("Unknown configuration key: {}", key)),
            }

            settings.save()?;

            println!("{}", format!("✓ Updated {} to: {}", key, value).green().bold());
            println!();
            println!("{}", "Configuration saved successfully!".green());

            Ok(())
        }

        ConfigAction::Validate => {
            println!("{}", "Validating configuration...".cyan().bold());
            println!();

            let settings = Settings::load()?;

            // Test Jira connection
            print!("{}", "  Testing Jira connection... ".dimmed());
            std::io::Write::flush(&mut std::io::stdout())?;

            let jira = api::jira::JiraClient::new(
                settings.jira.url.clone(),
                settings.jira.email.clone(),
                settings.jira.auth_method.clone(),
            );

            match jira.search_with_jql(&format!("project = {}", settings.jira.project_key), 1).await {
                Ok(_) => {
                    println!("{}", "✓".green().bold());
                }
                Err(e) => {
                    println!("{}", "✗".red().bold());
                    println!();
                    println!("{}", format!("  Jira connection failed: {}", e).red());
                    println!();
                    println!("{}", "  To fix:".yellow());
                    println!("{}", "    1. Check your Jira URL is correct".dimmed());
                    println!("{}", "    2. Verify your authentication token is valid".dimmed());
                    println!("{}", "    3. Update with: devflow config set jira.token <new-token>".dimmed());
                    return Err(anyhow::anyhow!("Jira validation failed"));
                }
            }

            // Test Git token (basic check)
            print!("{}", "  Checking Git token... ".dimmed());
            std::io::Write::flush(&mut std::io::stdout())?;

            if settings.git.token.is_empty() {
                println!("{}", "✗".red().bold());
                println!();
                println!("{}", "  Git token is empty".red());
                return Err(anyhow::anyhow!("Git token validation failed"));
            } else {
                println!("{}", "✓".green().bold());
            }

            println!();
            println!("{}", "✓ All validations passed!".green().bold());

            Ok(())
        }

        ConfigAction::Path => {
            let config_path = Settings::config_dir()?.join("config.toml");
            println!("{}", config_path.display());
            Ok(())
        }
    }
}

async fn handle_test_jira(
    ticket_id: &str,
    url: &str,
    email: &str,
    token: &str,
) -> anyhow::Result<()> {
    use colored::*;
    use config::settings::AuthMethod;

    println!("{}", "Testing Jira API connection...".cyan());
    println!();

    let jira = api::jira::JiraClient::new(
        url.to_string(),
        email.to_string(),
        AuthMethod::ApiToken {
            token: token.to_string(),
        },
    );

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

    #[test]
    fn test_open_jira_url_generation() {
        let jira_url = "https://jira.example.com";
        let ticket_id = "WAB-1234";
        let expected = format!("{}/browse/{}", jira_url, ticket_id);
        assert_eq!(expected, "https://jira.example.com/browse/WAB-1234");
    }

    #[test]
    fn test_open_board_url_generation() {
        let jira_url = "https://jira.example.com";
        let project_key = "WAB";
        let expected = format!("{}/jira/software/projects/{}/boards", jira_url, project_key);
        assert_eq!(expected, "https://jira.example.com/jira/software/projects/WAB/boards");
    }

    #[test]
    fn test_open_github_pr_url_generation() {
        let base_url = "https://api.github.com";
        let owner = "testuser";
        let repo = "testrepo";
        let branch = "feat/WAB-1234/test";
        let expected = format!("{}/{}/{}/pulls?q=is%3Apr+head%3A{}",
            base_url.replace("api.", ""),
            owner,
            repo,
            urlencoding::encode(branch)
        );
        assert_eq!(expected, "https://github.com/testuser/testrepo/pulls?q=is%3Apr+head%3Afeat%2FWAB-1234%2Ftest");
    }

    #[test]
    fn test_open_gitlab_mr_url_generation() {
        let base_url = "https://git.example.com";
        let branch = "feat/WAB-1234/test";
        let expected = format!("{}/merge_requests?scope=all&state=opened&source_branch={}",
            base_url,
            urlencoding::encode(branch)
        );
        assert_eq!(expected, "https://git.example.com/merge_requests?scope=all&state=opened&source_branch=feat%2FWAB-1234%2Ftest");
    }
}
