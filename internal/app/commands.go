package app

import (
	"bufio"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"github.com/Ilia01/devflow/internal/config"
	"github.com/Ilia01/devflow/internal/git"
	"github.com/Ilia01/devflow/internal/jira"
	"github.com/Ilia01/devflow/internal/models"
	githubProvider "github.com/Ilia01/devflow/internal/providers/github"
	gitlabProvider "github.com/Ilia01/devflow/internal/providers/gitlab"
	"github.com/Ilia01/devflow/internal/utils"
)

type jiraService interface {
	GetTicket(string) (*models.JiraTicket, error)
	UpdateStatus(string, string) error
	SearchWithJQL(string, int) ([]models.JiraTicket, error)
	TestConnection() error
}

type gitHubService interface {
	CreatePullRequest(sourceBranch, targetBranch, title, description string) (string, error)
}

type gitLabService interface {
	CreateMergeRequest(projectPath, sourceBranch, targetBranch, title, description string) (string, error)
}

var (
	jiraFactory = func(url, email string, auth config.AuthMethod) jiraService {
		return jira.NewClient(url, email, auth)
	}

	gitHubFactory = func(owner, repo, token string) gitHubService {
		return githubProvider.NewClient(owner, repo, token)
	}

	gitLabFactory = func(baseURL, token string) gitLabService {
		return gitlabProvider.NewClient(baseURL, token)
	}
)

func handleInit() error {
	fmt.Println(utils.Cyan(utils.Bold("DevFlow Configuration Setup")))
	fmt.Println()
	fmt.Println(utils.Dim("This will store your credentials in ~/.devflow/config.toml"))
	fmt.Println(utils.Dim("The file will be created with read-only permissions (600)"))
	fmt.Println()

	jiraURL, err := utils.Prompt("Jira URL (e.g., https://jira.<company>.com)")
	if err != nil {
		return err
	}
	jiraEmail, err := utils.Prompt("Jira email")
	if err != nil {
		return err
	}

	fmt.Println()
	fmt.Println(utils.Bold("Select authentication method:"))
	fmt.Println(utils.Dim("  1. Personal Access Token (for Jira Data Center/Server)"))
	fmt.Println(utils.Dim("  2. API Token (for Jira Cloud)"))
	authChoice, err := utils.PromptWithDefault("Choice (1/2)", "2")
	if err != nil {
		return err
	}

	var auth config.AuthMethod
	if authChoice == "1" {
		fmt.Println()
		fmt.Println(utils.Dim("To create a Personal Access Token:"))
		fmt.Println(utils.Dim("  1. Go to Jira → Profile → Personal Access Tokens"))
		fmt.Println(utils.Dim("  2. Click 'Create token'"))
		fmt.Println(utils.Dim("  3. Copy and paste it here"))
		token, err := utils.PromptPassword("Personal Access Token")
		if err != nil {
			return err
		}
		auth = config.AuthMethod{Type: "personal_access_token", Token: token}
	} else {
		fmt.Println()
		fmt.Println(utils.Dim("To create a Jira API token:"))
		fmt.Println(utils.Dim("  1. Go to https://id.atlassian.com/manage-profile/security/api-tokens"))
		fmt.Println(utils.Dim("  2. Click 'Create API token'"))
		fmt.Println(utils.Dim("  3. Copy and paste it here"))
		token, err := utils.PromptPassword("Jira API token")
		if err != nil {
			return err
		}
		auth = config.AuthMethod{Type: "api_token", Token: token}
	}

	projectKey, err := utils.Prompt("Default project key (e.g., WAB)")
	if err != nil {
		return err
	}

	fmt.Println()
	fmt.Println(utils.Bold("=== Git Configuration ==="))
	gitProvider, err := utils.PromptWithDefault("Git provider (gitlab/github)", "gitlab")
	if err != nil {
		return err
	}
	gitProvider = strings.ToLower(strings.TrimSpace(gitProvider))

	var gitBaseURL, gitOwner, gitRepo string
	if gitProvider == "github" {
		fmt.Println()
		fmt.Println(utils.Dim("For GitHub, create a token at:"))
		fmt.Println(utils.Dim("  Settings > Developer settings > Personal access tokens"))
		fmt.Println(utils.Dim("  Required scope: repo (full control)"))
		owner, err := utils.Prompt("Repository owner (username or org)")
		if err != nil {
			return err
		}
		repo, err := utils.Prompt("Repository name")
		if err != nil {
			return err
		}
		gitBaseURL = "https://api.github.com"
		gitOwner = owner
		gitRepo = repo
	} else {
		url, err := utils.Prompt("GitLab base URL (e.g., https://git.<company>.com)")
		if err != nil {
			return err
		}
		fmt.Println()
		fmt.Println(utils.Dim("For GitLab, create a token at:"))
		fmt.Println(utils.Dim("  Settings > Access Tokens"))
		fmt.Println(utils.Dim("  Required scopes: api"))
		gitBaseURL = url
	}

	gitToken, err := utils.PromptPassword("Git API token")
	if err != nil {
		return err
	}

	fmt.Println()
	fmt.Println(utils.Bold("=== Preferences ==="))
	branchPrefix, err := utils.PromptWithDefault("Branch prefix (feat/fix/test)", "feat")
	if err != nil {
		return err
	}
	defaultTransition, err := utils.PromptWithDefault("Default Jira transition", "In Progress")
	if err != nil {
		return err
	}

	settings := &config.Settings{
		Jira: config.JiraConfig{
			URL:        strings.TrimSpace(jiraURL),
			Email:      strings.TrimSpace(jiraEmail),
			ProjectKey: strings.TrimSpace(projectKey),
			AuthMethod: auth,
		},
		Git: config.GitConfig{
			Provider: gitProvider,
			BaseURL:  strings.TrimSpace(gitBaseURL),
			Token:    strings.TrimSpace(gitToken),
			Owner:    strings.TrimSpace(gitOwner),
			Repo:     strings.TrimSpace(gitRepo),
		},
		Preferences: config.Preferences{
			BranchPrefix:      strings.TrimSpace(branchPrefix),
			DefaultTransition: strings.TrimSpace(defaultTransition),
		},
	}

	if err := settings.Save(); err != nil {
		return err
	}

	configPath, err := config.ConfigPath()
	if err != nil {
		return err
	}

	fmt.Println()
	fmt.Println(utils.Green(utils.Bold("Configuration saved!")))
	fmt.Printf("  Location: %s\n\n", utils.BrightWhite(configPath))

	fmt.Println(utils.Cyan("Validating configuration..."))
	fmt.Println()

	fmt.Print(utils.Dim("  Testing Jira connection... "))
	jiraClient := jiraFactory(settings.Jira.URL, settings.Jira.Email, settings.Jira.AuthMethod)
	if err := jiraClient.TestConnection(); err != nil {
		fmt.Println(utils.Red("✗"))
		fmt.Println()
		fmt.Printf("  %s %v\n", utils.Yellow("Warning:"), err)
		fmt.Println(utils.Dim("  This may be expected if VPN/network restrictions apply."))
	} else {
		fmt.Println(utils.Green("✓"))
	}

	fmt.Print(utils.Dim("  Checking Git token... "))
	if settings.Git.Token == "" {
		fmt.Println(utils.Red("✗"))
		fmt.Println(utils.Yellow("  Warning: Git token is empty"))
	} else {
		fmt.Println(utils.Green("✓"))
	}

	fmt.Println()
	fmt.Println(utils.Green(utils.Bold("Setup complete!")))
	fmt.Println(utils.Yellow("Keep your API tokens secure!"))
	fmt.Println(utils.Dim("  Never commit config.toml to git"))

	return nil
}

func handleStart(ticketID string) error {
	settings, err := loadSettings()
	if err != nil {
		return err
	}

	gitClient, err := git.NewClient()
	if err != nil {
		return err
	}

	if branch, err := gitClient.CurrentBranch(); err == nil {
		if strings.Contains(strings.ToUpper(branch), strings.ToUpper(ticketID)) {
			fmt.Println(utils.Yellow(fmt.Sprintf("Already on branch: %s", branch)))
			fmt.Println(utils.Dim("Run 'devflow status' to see current state"))
			return nil
		}
	}

	fmt.Println(utils.Cyan(utils.Bold(fmt.Sprintf("Starting work on %s...", ticketID))))
	fmt.Println()
	fmt.Println(utils.Dim("  Fetching Jira ticket..."))

	jiraClient := jiraFactory(settings.Jira.URL, settings.Jira.Email, settings.Jira.AuthMethod)
	ticket, err := jiraClient.GetTicket(ticketID)
	if err != nil {
		return err
	}

	fmt.Println(utils.Green(fmt.Sprintf("  ✓ Found: %s", ticket.Fields.Summary)))
	fmt.Println(utils.Dim(fmt.Sprintf("    Status: %s", ticket.Fields.Status.Name)))

	prefix := settings.Preferences.BranchPrefix
	if prefix == "" {
		prefix = "feat"
	}
	branchName := utils.FormatBranchName(prefix, ticketID, ticket.Fields.Summary)

	fmt.Println()
	fmt.Println(utils.Cyan(fmt.Sprintf("  Creating branch: %s", branchName)))
	if err := gitClient.CreateBranch(branchName); err != nil {
		return err
	}

	transition := settings.Preferences.DefaultTransition
	if transition != "" {
		fmt.Println(utils.Cyan(fmt.Sprintf("  Updating Jira status to '%s'...", transition)))
		if err := jiraClient.UpdateStatus(ticketID, transition); err != nil {
			fmt.Println(utils.Yellow(fmt.Sprintf("  Could not update status: %v", err)))
		} else {
			fmt.Println(utils.Green(fmt.Sprintf("  ✓ Status updated to '%s'", transition)))
		}
	}

	fmt.Println()
	fmt.Println(utils.Green(utils.Bold("✨ All set! You're ready to code!")))
	fmt.Println()
	fmt.Printf("  %s %s\n", utils.Bold("Ticket:"), utils.BrightWhite(ticketID))
	fmt.Printf("  %s %s\n", utils.Bold("Branch:"), utils.BrightWhite(branchName))
	fmt.Printf("  %s %s\n", utils.Bold("Summary:"), utils.Dim(ticket.Fields.Summary))

	return nil
}

func handleStatus() error {
	fmt.Println(utils.Cyan("Current Status"))
	fmt.Println()

	gitClient, err := git.NewClient()
	if err != nil {
		fmt.Println(utils.Yellow("  Not in a git repository"))
		fmt.Println(utils.Dim(fmt.Sprintf("  %v", err)))
		return nil
	}

	if branch, err := gitClient.CurrentBranch(); err == nil {
		fmt.Printf("  %s %s\n", utils.Bold("Branch:"), utils.BrightWhite(branch))
	} else {
		fmt.Printf("  %s %s\n", utils.Bold("Branch:"), utils.Red(err.Error()))
	}

	if summary, err := gitClient.StatusSummary(); err == nil {
		fmt.Printf("\n  %s:\n%s\n", utils.Bold("Status"), summary)
	} else {
		fmt.Printf("  %s %s\n", utils.Bold("Status:"), utils.Red(err.Error()))
	}

	return nil
}
func handleList(statusFilter, projectFilter string, jsonOutput bool) error {
	settings, err := loadSettings()
	if err != nil {
		return err
	}

	jiraClient := jiraFactory(settings.Jira.URL, settings.Jira.Email, settings.Jira.AuthMethod)

	project := strings.TrimSpace(projectFilter)
	if project == "" {
		project = settings.Jira.ProjectKey
	}

	jqlParts := []string{"assignee = currentUser()"}
	if project != "" {
		jqlParts = append(jqlParts, fmt.Sprintf("project = %s", project))
	}
	if statusFilter != "" {
		jqlParts = append(jqlParts, fmt.Sprintf("status = \"%s\"", statusFilter))
	}

	jql := strings.Join(jqlParts, " AND ")
	tickets, err := jiraClient.SearchWithJQL(jql, 50)
	if err != nil {
		return err
	}

	if jsonOutput {
		data, err := json.MarshalIndent(tickets, "", "  ")
		if err != nil {
			return err
		}
		fmt.Println(string(data))
		return nil
	}

	fmt.Println(utils.Cyan(utils.Bold("Your Assigned Tickets")))
	fmt.Println()

	if len(tickets) == 0 {
		fmt.Println(utils.Dim("  No tickets assigned to you"))
		return nil
	}

	fmt.Printf("%s  %d tickets found\n\n", utils.Dim(""), len(tickets))
	printTicketList(tickets)

	return nil
}

type searchOptions struct {
	Query       string
	Assignee    string
	Status      string
	Project     string
	Limit       int
	Interactive bool
}

func handleSearch(opts searchOptions) error {
	settings, err := loadSettings()
	if err != nil {
		return err
	}

	jiraClient := jiraFactory(settings.Jira.URL, settings.Jira.Email, settings.Jira.AuthMethod)

	fmt.Println(utils.Cyan(utils.Bold(fmt.Sprintf("Searching for: \"%s\"", opts.Query))))
	fmt.Println()

	jqlParts := []string{fmt.Sprintf("(summary ~ \"%s\" OR description ~ \"%s\")", escapeJQL(opts.Query), escapeJQL(opts.Query))}

	project := strings.TrimSpace(opts.Project)
	if project == "" {
		project = settings.Jira.ProjectKey
	}
	if project != "" {
		jqlParts = append(jqlParts, fmt.Sprintf("project = %s", project))
	}

	if opts.Assignee != "" {
		if opts.Assignee == "me" {
			jqlParts = append(jqlParts, "assignee = currentUser()")
		} else {
			jqlParts = append(jqlParts, fmt.Sprintf("assignee = \"%s\"", opts.Assignee))
		}
	}

	if opts.Status != "" {
		jqlParts = append(jqlParts, fmt.Sprintf("status = \"%s\"", opts.Status))
	}

	jql := strings.Join(jqlParts, " AND ")
	fmt.Println(utils.Dim(fmt.Sprintf("  JQL: %s", jql)))
	fmt.Println()

	tickets, err := jiraClient.SearchWithJQL(jql, opts.Limit)
	if err != nil {
		return err
	}

	if len(tickets) == 0 {
		fmt.Println(utils.Dim("  No tickets found"))
		return nil
	}

	for i, ticket := range tickets {
		fmt.Printf("  %s. %s [%s]  %s\n",
			utils.Dim(strconv.Itoa(i+1)),
			utils.BrightWhite(ticket.Key),
			colorStatus(ticket.Fields.Status.Name),
			ticket.Fields.Summary,
		)
	}

	if len(tickets) == opts.Limit {
		fmt.Println()
		fmt.Println(utils.Dim(fmt.Sprintf("  Showing %d of potentially more results. Use --limit to see more.", opts.Limit)))
	}

	if opts.Interactive {
		idx, err := promptSelection(len(tickets))
		if err != nil {
			return err
		}
		if idx >= 0 {
			selected := tickets[idx]
			fmt.Println()
			fmt.Println(utils.Cyan(utils.Bold(fmt.Sprintf("Starting work on %s...", selected.Key))))
			return handleStart(selected.Key)
		}
		fmt.Println()
		fmt.Println(utils.Yellow("No ticket selected"))
	}

	return nil
}

func handleOpen(ticketID string, openPR, openBoard bool) error {
	settings, err := loadSettings()
	if err != nil {
		return err
	}

	if openBoard {
		boardURL := fmt.Sprintf("%s/jira/software/projects/%s/boards", settings.Jira.URL, settings.Jira.ProjectKey)
		fmt.Printf("%s %s\n", utils.Dim("Opening board:"), utils.BrightWhite(boardURL))
		return utils.OpenURL(boardURL)
	}

	if ticketID == "" {
		gitClient, err := git.NewClient()
		if err != nil {
			return err
		}
		branch, err := gitClient.CurrentBranch()
		if err != nil {
			return err
		}
		ticketID, err = utils.ExtractTicketID(branch)
		if err != nil {
			return err
		}
	}

	if openPR {
		gitClient, err := git.NewClient()
		if err != nil {
			return err
		}
		branch, err := gitClient.CurrentBranch()
		if err != nil {
			return err
		}

		provider := strings.ToLower(settings.Git.Provider)
		switch provider {
		case "github":
			if settings.Git.Owner == "" || settings.Git.Repo == "" {
				return errors.New("GitHub owner/repo not configured")
			}
			base := strings.Replace(strings.TrimSuffix(settings.Git.BaseURL, "/"), "api.", "", 1)
			prURL := fmt.Sprintf("%s/%s/%s/pulls?q=is%%3Apr+head%%3A%s", base, settings.Git.Owner, settings.Git.Repo, url.QueryEscape(branch))
			fmt.Printf("%s %s\n", utils.Dim("Opening PR:"), utils.BrightWhite(prURL))
			return utils.OpenURL(prURL)
		case "gitlab":
			prURL := fmt.Sprintf("%s/merge_requests?scope=all&state=opened&source_branch=%s", strings.TrimSuffix(settings.Git.BaseURL, "/"), url.QueryEscape(branch))
			fmt.Printf("%s %s\n", utils.Dim("Opening MR:"), utils.BrightWhite(prURL))
			return utils.OpenURL(prURL)
		default:
			return fmt.Errorf("unsupported git provider: %s", provider)
		}
	}

	ticketURL := fmt.Sprintf("%s/browse/%s", settings.Jira.URL, ticketID)
	fmt.Printf("%s %s\n", utils.Dim("Opening ticket:"), utils.BrightWhite(ticketURL))
	return utils.OpenURL(ticketURL)
}

func handleCommit(message string) error {
	settings, err := loadSettings()
	if err != nil {
		return err
	}

	gitClient, err := git.NewClient()
	if err != nil {
		return err
	}
	branch, err := gitClient.CurrentBranch()
	if err != nil {
		return err
	}
	ticketID, err := utils.ExtractTicketID(branch)
	if err != nil {
		return err
	}

	formatted := fmt.Sprintf("%s\n\n%s: %s/browse/%s", message, ticketID, settings.Jira.URL, ticketID)
	if err := gitClient.Commit(formatted); err != nil {
		return err
	}

	fmt.Println()
	fmt.Println(utils.Green(utils.Bold("Commit created successfully!")))
	fmt.Printf("  %s %s\n", utils.Bold("Message:"), message)
	fmt.Printf("  %s %s\n", utils.Bold("Ticket:"), utils.BrightWhite(ticketID))

	return nil
}

func handleDone() error {
	settings, err := loadSettings()
	if err != nil {
		return err
	}

	gitClient, err := git.NewClient()
	if err != nil {
		return err
	}

	clean, err := gitClient.IsClean()
	if err != nil {
		return err
	}
	if !clean {
		return errors.New("uncommitted changes detected. Commit or stash before running 'devflow done'")
	}

	branch, err := gitClient.CurrentBranch()
	if err != nil {
		return err
	}
	ticketID, err := utils.ExtractTicketID(branch)
	if err != nil {
		return err
	}

	fmt.Println(utils.Cyan(utils.Bold("Finalizing work...")))
	fmt.Println()
	fmt.Println(utils.Dim("  Pushing branch to remote..."))
	if err := gitClient.Push(branch); err != nil {
		return err
	}

	jiraClient := jiraFactory(settings.Jira.URL, settings.Jira.Email, settings.Jira.AuthMethod)
	fmt.Println(utils.Dim("  Fetching ticket information..."))
	ticket, err := jiraClient.GetTicket(ticketID)
	if err != nil {
		return err
	}

	prTitle := fmt.Sprintf("%s: %s", ticketID, ticket.Fields.Summary)
	prDescription := fmt.Sprintf("Resolves %s\n\nJira: %s/browse/%s", ticketID, settings.Jira.URL, ticketID)

	provider := strings.ToLower(settings.Git.Provider)
	var prURL string
	switch provider {
	case "github":
		if settings.Git.Owner == "" || settings.Git.Repo == "" {
			return errors.New("GitHub owner/repo not configured")
		}
		fmt.Println(utils.Dim("  Creating pull request..."))
		client := gitHubFactory(settings.Git.Owner, settings.Git.Repo, settings.Git.Token)
		prURL, err = client.CreatePullRequest(branch, "main", prTitle, prDescription)
	case "gitlab":
		fmt.Println(utils.Dim("  Creating merge request..."))
		projectPath := filepath.Base(gitClient.Root())
		client := gitLabFactory(settings.Git.BaseURL, settings.Git.Token)
		prURL, err = client.CreateMergeRequest(projectPath, branch, "main", prTitle, prDescription)
	default:
		return fmt.Errorf("unsupported git provider: %s", provider)
	}
	if err != nil {
		return err
	}

	fmt.Println(utils.Dim("  Updating Jira status to 'In Review'..."))
	if err := jiraClient.UpdateStatus(ticketID, "In Review"); err != nil {
		fmt.Println(utils.Yellow(fmt.Sprintf("  Could not update status: %v", err)))
	} else {
		fmt.Println(utils.Green("  ✓ Status updated to 'In Review'"))
	}

	label := "PR"
	if provider == "gitlab" {
		label = "MR"
	}

	fmt.Println()
	fmt.Println(utils.Green(utils.Bold("All done! Ready for review!")))
	fmt.Printf("  %s %s\n", utils.Bold("Ticket:"), utils.BrightWhite(ticketID))
	fmt.Printf("  %s %s\n", utils.Bold("Branch:"), utils.BrightWhite(branch))
	fmt.Printf("  %s %s\n", utils.Bold(label+":"), utils.Cyan(prURL))

	return nil
}

func handleConfigShow() error {
	settings, err := loadSettings()
	if err != nil {
		return err
	}
	printConfig(settings)
	return nil
}

func handleConfigSet(key, value string) error {
	settings, err := loadSettings()
	if err != nil {
		return err
	}
	if err := updateConfigValue(settings, key, value); err != nil {
		return err
	}
	if err := settings.Save(); err != nil {
		return err
	}
	fmt.Println(utils.Green(utils.Bold(fmt.Sprintf("✓ Updated %s to: %s", key, value))))
	return nil
}

func handleConfigValidate() error {
	settings, err := loadSettings()
	if err != nil {
		return err
	}
	fmt.Println(utils.Cyan(utils.Bold("Validating configuration...")))
	fmt.Println()
	fmt.Print(utils.Dim("  Testing Jira connection... "))
	jiraClient := jiraFactory(settings.Jira.URL, settings.Jira.Email, settings.Jira.AuthMethod)
	if err := jiraClient.TestConnection(); err != nil {
		fmt.Println(utils.Red("✗"))
		fmt.Println(utils.Yellow(fmt.Sprintf("  Jira validation failed: %v", err)))
	} else {
		fmt.Println(utils.Green("✓"))
	}
	if settings.Git.Token == "" {
		fmt.Println(utils.Yellow("  Warning: Git token is empty"))
	} else {
		fmt.Println(utils.Dim("  Git token configured"))
	}
	return nil
}

func handleConfigPath() error {
	path, err := config.ConfigPath()
	if err != nil {
		return err
	}
	fmt.Println(path)
	return nil
}

func handleTestJira(ticketID, jiraURL, email, token string) error {
	if ticketID == "" || jiraURL == "" || email == "" || token == "" {
		return errors.New("ticket id, url, email, and token are required")
	}

	jiraClient := jiraFactory(jiraURL, email, config.AuthMethod{Type: "api_token", Token: token})

	fmt.Println(utils.Cyan("Testing Jira API connection..."))
	fmt.Println()
	fmt.Println(utils.Dim(fmt.Sprintf("  Fetching ticket %s...", ticketID)))

	ticket, err := jiraClient.GetTicket(ticketID)
	if err != nil {
		return err
	}

	fmt.Println()
	fmt.Println(utils.Green(utils.Bold("✓ Successfully fetched ticket!")))
	fmt.Println()
	fmt.Printf("  %s %s\n", utils.Bold("Key:"), utils.BrightWhite(ticket.Key))
	fmt.Printf("  %s %s\n", utils.Bold("Summary:"), ticket.Fields.Summary)
	fmt.Printf("  %s %s\n", utils.Bold("Status:"), utils.Yellow(ticket.Fields.Status.Name))
	if ticket.Fields.Assignee != nil {
		fmt.Printf("  %s %s\n", utils.Bold("Assignee:"), ticket.Fields.Assignee.DisplayName)
	}

	return nil
}

func loadSettings() (*config.Settings, error) {
	settings, err := config.Load()
	if err != nil {
		if errors.Is(err, config.ErrConfigNotFound) {
			return nil, errors.New("configuration not found. Run 'devflow init' first")
		}
		return nil, err
	}
	return settings, nil
}

func printConfig(settings *config.Settings) {
	fmt.Println(utils.Cyan(utils.Bold("Current Configuration")))
	fmt.Println()

	fmt.Println(utils.Bold("[jira]"))
	fmt.Printf("  %s %s\n", utils.Dim("url:"), utils.BrightWhite(settings.Jira.URL))
	fmt.Printf("  %s %s\n", utils.Dim("email:"), utils.BrightWhite(settings.Jira.Email))
	fmt.Printf("  %s %s\n", utils.Dim("project_key:"), utils.BrightWhite(settings.Jira.ProjectKey))
	fmt.Printf("  %s %s\n", utils.Dim("auth_method:"), utils.BrightWhite(settings.Jira.AuthMethod.Type))
	fmt.Printf("  %s %s\n", utils.Dim("token:"), utils.Yellow(config.MaskToken(settings.Jira.AuthMethod.Token)))

	fmt.Println()
	fmt.Println(utils.Bold("[git]"))
	fmt.Printf("  %s %s\n", utils.Dim("provider:"), utils.BrightWhite(settings.Git.Provider))
	fmt.Printf("  %s %s\n", utils.Dim("base_url:"), utils.BrightWhite(settings.Git.BaseURL))
	if settings.Git.Owner != "" {
		fmt.Printf("  %s %s\n", utils.Dim("owner:"), utils.BrightWhite(settings.Git.Owner))
	}
	if settings.Git.Repo != "" {
		fmt.Printf("  %s %s\n", utils.Dim("repo:"), utils.BrightWhite(settings.Git.Repo))
	}
	fmt.Printf("  %s %s\n", utils.Dim("token:"), utils.Yellow(config.MaskToken(settings.Git.Token)))

	fmt.Println()
	fmt.Println(utils.Bold("[preferences]"))
	fmt.Printf("  %s %s\n", utils.Dim("branch_prefix:"), utils.BrightWhite(settings.Preferences.BranchPrefix))
	fmt.Printf("  %s %s\n", utils.Dim("default_transition:"), utils.BrightWhite(settings.Preferences.DefaultTransition))
}

func updateConfigValue(settings *config.Settings, key, value string) error {
	parts := strings.Split(key, ".")
	if len(parts) != 2 {
		return errors.New("invalid key format. Use section.field (e.g., jira.email)")
	}

	section, field := parts[0], parts[1]
	switch section {
	case "jira":
		switch field {
		case "url":
			settings.Jira.URL = value
		case "email":
			settings.Jira.Email = value
		case "project_key":
			settings.Jira.ProjectKey = value
		case "token":
			settings.Jira.AuthMethod.Token = value
		case "auth_method":
			settings.Jira.AuthMethod.Type = value
		default:
			return fmt.Errorf("unknown jira field: %s", field)
		}
	case "git":
		switch field {
		case "provider":
			settings.Git.Provider = value
		case "base_url":
			settings.Git.BaseURL = value
		case "token":
			settings.Git.Token = value
		case "owner":
			settings.Git.Owner = value
		case "repo":
			settings.Git.Repo = value
		default:
			return fmt.Errorf("unknown git field: %s", field)
		}
	case "preferences":
		switch field {
		case "branch_prefix":
			settings.Preferences.BranchPrefix = value
		case "default_transition":
			settings.Preferences.DefaultTransition = value
		default:
			return fmt.Errorf("unknown preferences field: %s", field)
		}
	default:
		return fmt.Errorf("unknown configuration section: %s", section)
	}

	return nil
}

func printTicketList(tickets []models.JiraTicket) {
	for _, ticket := range tickets {
		fmt.Printf("  %s [%s]  %s\n",
			utils.Bold(utils.BrightWhite(ticket.Key)),
			colorStatus(ticket.Fields.Status.Name),
			ticket.Fields.Summary,
		)
	}
}

func colorStatus(status string) string {
	switch status {
	case "In Progress":
		return utils.Green(status)
	case "To Do":
		return utils.Yellow(status)
	case "In Review", "Code Review":
		return utils.Blue(status)
	case "Done":
		return utils.Dim(status)
	default:
		return status
	}
}

func promptSelection(count int) (int, error) {
	if count == 0 {
		return -1, nil
	}
	reader := bufio.NewReader(os.Stdin)
	fmt.Print("Select a ticket number (or press Enter to cancel): ")
	input, err := reader.ReadString('\n')
	if err != nil {
		return -1, err
	}
	input = strings.TrimSpace(input)
	if input == "" {
		return -1, nil
	}
	idx, err := strconv.Atoi(input)
	if err != nil {
		return -1, fmt.Errorf("invalid selection: %s", input)
	}
	if idx < 1 || idx > count {
		return -1, fmt.Errorf("selection out of range (1-%d)", count)
	}
	return idx - 1, nil
}

func escapeJQL(value string) string {
	return strings.ReplaceAll(value, "\"", "\\\"")
}
