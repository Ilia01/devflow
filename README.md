# DevFlow

A blazing-fast CLI tool for automating Jira/Git workflows. Stop context switching between Jira, Git, and GitHub/GitLab - manage your entire development workflow from the command line.

## Features

- **Integrated Workflow**: One command to fetch Jira tickets, create branches, and update statuses
- **Smart Branch Naming**: Automatically generates clean branch names from ticket summaries
- **Automatic Commit Formatting**: Links commits to Jira tickets automatically
- **PR/MR Automation**: Push, create pull/merge requests, and update Jira status in one command
- **Fast & Lightweight**: Written in Rust, single binary, no runtime dependencies
- **Universal**: Works with any Jira instance, GitHub, and GitLab

## Installation

### From Source

```bash
git clone https://github.com/Ilia01/devflow.git
cd devflow
cargo install --path .
```

Make sure `~/.cargo/bin` is in your PATH:
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

## Quick Start

### 1. Initialize Configuration

```bash
devflow init
```

This will prompt you for:
- Jira URL and credentials (email + API token)
- GitLab URL and access token
- Workflow preferences (branch prefix, default transition)

Configuration is stored securely at `~/.devflow/config.toml` with 600 permissions.

### 2. Start Working on a Ticket

```bash
devflow start WAB-1234
```

This will:
- Fetch the ticket from Jira
- Create a branch: `feat/WAB-1234/ticket_summary`
- Update Jira status to "In Progress"

### 3. Make Commits

```bash
devflow commit "Add user authentication"
```

This creates a commit with automatic ticket reference:
```
Add user authentication

WAB-1234: https://jira.company.com/browse/WAB-1234
```

### 4. Finish and Create MR

```bash
devflow done
```

This will:
- Push your branch to remote
- Create a pull request (GitHub) or merge request (GitLab)
- Update Jira status to "In Review"
- Display the PR/MR URL

### 5. List Your Assigned Tickets

```bash
devflow list
```

Shows all Jira tickets assigned to you in the current project.

### 6. Check Current Status

```bash
devflow status
```

Shows your current branch and working directory status.

## Configuration

Configuration file location: `~/.devflow/config.toml`

**For GitLab:**
```toml
[jira]
url = "https://jira.company.com"
email = "you@company.com"
api_token = "your-api-token"
project_key = "WAB"

[git]
provider = "gitlab"
base_url = "https://git.company.com"
token = "your-gitlab-token"

[preferences]
branch_prefix = "feat"
default_transition = "In Progress"
```

**For GitHub:**
```toml
[jira]
url = "https://jira.company.com"
email = "you@company.com"
api_token = "your-api-token"
project_key = "PROJ"

[git]
provider = "github"
base_url = "https://api.github.com"
token = "your-github-token"
owner = "your-username"
repo = "your-repo"

[preferences]
branch_prefix = "feat"
default_transition = "In Progress"
```

### Getting API Tokens

**Jira API Token:**
1. Go to https://id.atlassian.com/manage-profile/security/api-tokens
2. Click "Create API token"
3. Copy and use in `devflow init`

**GitLab Access Token:**
1. Go to GitLab → Settings → Access Tokens
2. Create token with `api` scope
3. Copy and use in `devflow init`

**GitHub Personal Access Token:**
1. Go to GitHub → Settings → Developer settings → Personal access tokens → Generate new token
2. Select `repo` scope (full control of private repositories)
3. Copy and use in `devflow init`

## Commands

| Command | Description |
|---------|-------------|
| `devflow init` | Set up configuration and credentials |
| `devflow start <ticket>` | Start work on a Jira ticket |
| `devflow status` | Show current branch and git status |
| `devflow list` | List all assigned Jira tickets |
| `devflow commit <message>` | Commit with automatic ticket reference |
| `devflow done` | Push, create MR, and update Jira |

## Branch Naming Convention

DevFlow automatically creates branch names from ticket summaries:

- Format: `{prefix}/{TICKET-ID}/{description}`
- Uses underscores for spaces
- Filters special characters
- Limits to 5 words maximum

Examples:
- "Add user authentication" → `feat/WAB-1234/add_user_authentication`
- "Fix bug: login doesn't work!" → `fix/PROJ-999/fix_bug_login_doesnt_work`

## Development

### Prerequisites

- Rust 1.70+
- Git 2.0+

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Project Structure

```
devflow/
├── src/
│   ├── main.rs           # CLI entry point and command handlers
│   ├── api/
│   │   ├── git.rs        # Git operations via libgit2
│   │   ├── jira.rs       # Jira REST API client
│   │   └── gitlab.rs     # GitLab API client
│   ├── config/
│   │   └── settings.rs   # Configuration management
│   └── models/
│       └── ticket.rs     # Jira ticket data structures
├── Cargo.toml
└── README.md
```

## Troubleshooting

### "Configuration not found" error
Run `devflow init` to set up your credentials.

### "Not in a git repository" error
Make sure you're inside a Git repository when running commands.

### SSH authentication fails
Ensure your SSH keys are set up and added to ssh-agent:
```bash
ssh-add ~/.ssh/id_rsa
```

### Jira API errors
- Verify your API token is valid
- Check that your email matches your Jira account
- Ensure you have permissions to update ticket statuses

## Security

- Credentials are stored in `~/.devflow/config.toml` with 600 permissions
- Never commit `config.toml` to version control
- API tokens are used instead of passwords
- SSH keys are used for Git operations

## Why DevFlow?

Built by a developer tired of context switching between Jira tabs, Git commands, and GitLab UI. DevFlow brings the entire workflow into your terminal where you already spend most of your time.

Inspired by tools like [ThePrimeagen](https://github.com/ThePrimeagen)'s workflow automation philosophy: automate repetitive tasks and stay in the terminal.

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Author

Built with Rust by [Ilia Goginashvili](https://github.com/Ilia01)