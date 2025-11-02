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
- Jira URL and email
- Authentication method:
  - **Personal Access Token** (for Jira Data Center/Server)
  - **API Token** (for Jira Cloud)
- Git provider (GitHub/GitLab) and access token
- Workflow preferences (branch prefix, default transition)

**Configuration Validation:** DevFlow automatically tests your Jira connection during setup to ensure credentials are valid before saving.

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
devflow list                           # All your tickets
devflow list --status "To Do"          # Filter by status
devflow list --project WAB             # Different project
devflow list --json                    # JSON output for scripting
```

Shows all Jira tickets assigned to you with optional filtering.

### 6. Open Ticket or PR in Browser

```bash
devflow open           # Opens current ticket in Jira
devflow open WAB-1234  # Opens specific ticket
devflow open --pr      # Opens PR/MR for current branch
devflow open --board   # Opens Jira board
```

Quick way to jump to tickets or pull requests without leaving the terminal.

### 7. Search Jira Tickets

```bash
devflow search "login bug"                       # Search by text
devflow search "auth" --assignee me              # My tickets matching "auth"
devflow search "API" --status "To Do"            # By status
devflow search "bug" --project WAB --limit 20    # Different project, more results
devflow search "auth" --interactive              # Interactive mode - select ticket to start work
devflow search "bug" -i                          # Short form of --interactive
```

Searches ticket summaries and descriptions with optional filters. Use `--interactive` to select a ticket and immediately start working on it.

### 8. Check Current Status

```bash
devflow status
```

Shows your current branch and working directory status.

## Configuration

Configuration file location: `~/.devflow/config.toml`

**For Jira Cloud with GitLab:**
```toml
[jira]
url = "https://your-company.atlassian.net"
email = "you@company.com"
project_key = "PROJ"

[jira.auth_method]
type = "api_token"
token = "your-api-token"

[git]
provider = "gitlab"
base_url = "https://git.company.com"
token = "your-gitlab-token"

[preferences]
branch_prefix = "feat"
default_transition = "In Progress"
```

**For Jira Data Center/Server with GitHub:**
```toml
[jira]
url = "https://jira.company.com"
email = "you@company.com"
project_key = "PROJ"

[jira.auth_method]
type = "personal_access_token"
token = "your-personal-access-token"

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

**Jira Personal Access Token (Data Center/Server):**
1. Go to your Jira instance → Profile → Personal Access Tokens
2. Click "Create token"
3. Give it a name and set expiration
4. Copy and use in `devflow init`

**Jira API Token (Cloud):**
1. Go to https://id.atlassian.com/manage-profile/security/api-tokens
2. Click "Create API token"
3. Give it a name
4. Copy and use in `devflow init`

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
| `devflow search <query>` | Search Jira tickets with filters |
| `devflow open [ticket]` | Open ticket or PR in browser |
| `devflow commit <message>` | Commit with automatic ticket reference |
| `devflow done` | Push, create MR, and update Jira |
| `devflow config <action>` | Manage configuration (show/set/validate/path) |

### Config Management

Instead of re-running `devflow init` to update your configuration, you can now use the config commands:

```bash
# View current configuration (with masked secrets)
devflow config show

# Update a specific value (e.g., if your Jira token expired)
devflow config set jira.token <new-token>
devflow config set jira.email <new-email>
devflow config set git.token <new-token>

# Validate your configuration by testing API connections
devflow config validate

# Get the path to your config file
devflow config path
```

Available config keys:
- `jira.url` - Your Jira instance URL
- `jira.email` - Your Jira email
- `jira.token` - Your Jira authentication token
- `jira.project_key` - Default project key
- `git.provider` - Git provider (github/gitlab)
- `git.base_url` - Git instance URL
- `git.token` - Git access token
- `git.owner` - GitHub repository owner
- `git.repo` - GitHub repository name
- `preferences.branch_prefix` - Default branch prefix
- `preferences.default_transition` - Default Jira transition

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

### Jira Data Center compatibility
DevFlow works with both Jira Cloud and Jira Data Center/Server instances. The tool automatically uses the correct API version (`/rest/api/latest/`) which works with Personal Access Tokens on Data Center.

If you encounter API errors:
- For Jira Data Center/Server: Use Personal Access Token authentication
- For Jira Cloud: Use API Token authentication
- You can override the API version with: `JIRA_API_VERSION=2 devflow list`

### Debug mode
For troubleshooting API issues, enable debug logging with the `--verbose` flag:
```bash
devflow --verbose list
```

Or use the environment variable:
```bash
DEVFLOW_DEBUG=1 devflow list
```

This shows detailed request/response information including:
- API URLs being called
- Request bodies (JQL queries, etc.)
- Response status codes
- Raw JSON responses (first 500 chars)
- Parsing errors with ticket data

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