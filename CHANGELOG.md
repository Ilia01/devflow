# Changelog

All notable changes to DevFlow will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.2.0] - 2025-11-02

### Added
- **Config Management Commands**
  - `devflow config show` - Display configuration with masked secrets
  - `devflow config set <key> <value>` - Update individual config values
  - `devflow config validate` - Test API connections without re-init
  - `devflow config path` - Show config file location
  - Config now saves even if validation fails (prevents data loss)

- **List Command Filters**
  - `devflow list --status "In Progress"` - Filter tickets by status
  - `devflow list --project WAB` - List tickets from different project
  - `devflow list --json` - JSON output for scripting/automation

- **Interactive Search**
  - `devflow search --interactive` - Select ticket from search results and immediately start work
  - Uses arrow keys to navigate, Enter to select

- **Debug Mode**
  - `--verbose` flag for debug output (shows API requests/responses)
  - Alternative: `DEVFLOW_DEBUG=1` environment variable
  - Shows API URLs, request bodies, response status, raw JSON

### Fixed
- **Jira Data Center Compatibility**
  - Changed default API version from `/rest/api/3/` to `/rest/api/latest/`
  - Fixes authentication issues with Jira Data Center search endpoint
  - Data Center `/rest/api/3/search` requires sessions, `/rest/api/latest/` accepts PAT
  - Added `JIRA_API_VERSION` env var to override API version if needed

- **Config Validation**
  - Config now saves before validation (prevents losing data on VPN/network errors)
  - Added helpful error messages for 403 Forbidden (VPN/network access issues)
  - Validation failures are now warnings, not fatal errors

### Changed
- Bumped version to 0.2.0
- Enhanced error handling with better context
- Improved debug logging throughout API calls

### Documentation
- Updated README with troubleshooting section for Jira Data Center
- Documented `--verbose` flag and `DEVFLOW_DEBUG` environment variable
- Added config management documentation
- Created comprehensive ROADMAP for v0.2.0 through v1.0.0

## [0.1.0] - 2025-10-31

### Added
- Initial release
- **Core Workflow Commands**
  - `devflow init` - Interactive configuration setup
  - `devflow start <ticket>` - Fetch ticket, create branch, update Jira status
  - `devflow commit <message>` - Commit with automatic ticket reference
  - `devflow done` - Push, create PR/MR, update Jira status
  - `devflow status` - Show current ticket and branch

- **Discovery Commands**
  - `devflow list` - List assigned Jira tickets
  - `devflow search <query>` - Search tickets with filters
  - `devflow open [ticket]` - Open ticket or PR in browser

- **Authentication**
  - Personal Access Token (PAT) support for Jira Data Center/Server
  - API Token support for Jira Cloud
  - Bearer token authentication for enterprise Jira
  - Basic auth for Jira Cloud

- **Git Provider Support**
  - GitHub integration (PR creation, status updates)
  - GitLab integration (MR creation, status updates)

- **Security**
  - Secure config storage at `~/.devflow/config.toml` with 600 permissions
  - Token masking in config display
  - Credentials never logged or exposed

- **Quality**
  - 46 unit tests with mocked HTTP responses
  - Custom error types with helpful messages
  - Comprehensive README with examples

### Technical Details
- Built with Rust
- Uses clap for CLI parsing
- reqwest for HTTP/API calls
- libgit2 for Git operations
- dialoguer for interactive prompts
- serde for serialization
