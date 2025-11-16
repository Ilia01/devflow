package app

import (
	"errors"
	"os"
	"strings"

	"github.com/spf13/cobra"
)

var (
	rootCmd = &cobra.Command{
		Use:           "devflow",
		Short:         "Automate your Jira/Git workflow",
		Long:          "DevFlow helps you manage Jira tickets and Git workflows from the terminal.",
		SilenceUsage:  true,
		SilenceErrors: true,
		PersistentPreRun: func(cmd *cobra.Command, args []string) {
			if verbose {
				os.Setenv("DEVFLOW_DEBUG", "1")
			}
		},
	}

	verbose bool

	initHandler       = handleInit
	startHandler      = handleStart
	statusHandler     = handleStatus
	listHandler       = handleList
	searchHandler     = handleSearch
	openHandler       = handleOpen
	commitHandler     = handleCommit
	doneHandler       = handleDone
	configShowHandler = handleConfigShow
	configSetHandler  = handleConfigSet
	configValHandler  = handleConfigValidate
	configPathHandler = handleConfigPath
	testJiraHandler   = handleTestJira
)

func Execute() error {
	return rootCmd.Execute()
}

func init() {
	rootCmd.PersistentFlags().BoolVarP(&verbose, "verbose", "v", false, "Enable verbose output")

	rootCmd.AddCommand(initCmd)
	rootCmd.AddCommand(startCmd)
	rootCmd.AddCommand(statusCmd)
	rootCmd.AddCommand(listCmd)
	rootCmd.AddCommand(searchCmd)
	rootCmd.AddCommand(openCmd)
	rootCmd.AddCommand(commitCmd)
	rootCmd.AddCommand(doneCmd)
	rootCmd.AddCommand(configCmd)
	rootCmd.AddCommand(testJiraCmd)
}

var initCmd = &cobra.Command{
	Use:   "init",
	Short: "Initialize DevFlow configuration",
	RunE: func(cmd *cobra.Command, args []string) error {
		return initHandler()
	},
}

var startCmd = &cobra.Command{
	Use:   "start <ticket-id>",
	Short: "Start work on a Jira ticket",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		return startHandler(args[0])
	},
}

var statusCmd = &cobra.Command{
	Use:   "status",
	Short: "Show current git status",
	RunE: func(cmd *cobra.Command, args []string) error {
		return statusHandler()
	},
}

var (
	listStatus  string
	listProject string
	listJSON    bool
)

var listCmd = &cobra.Command{
	Use:   "list",
	Short: "List assigned Jira tickets",
	RunE: func(cmd *cobra.Command, args []string) error {
		return listHandler(listStatus, listProject, listJSON)
	},
}

var (
	searchOpts = searchOptions{}
)

var searchCmd = &cobra.Command{
	Use:   "search <query>",
	Short: "Search Jira tickets",
	Args:  cobra.MinimumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		searchOpts.Query = strings.Join(args, " ")
		if searchOpts.Limit <= 0 {
			searchOpts.Limit = 10
		}
		return searchHandler(searchOpts)
	},
}

var (
	openTicket string
	openPR     bool
	openBoard  bool
)

var openCmd = &cobra.Command{
	Use:   "open [ticket-id]",
	Short: "Open ticket or PR/MR in browser",
	Args:  cobra.RangeArgs(0, 1),
	RunE: func(cmd *cobra.Command, args []string) error {
		ticketID := openTicket
		if ticketID == "" && len(args) > 0 {
			ticketID = args[0]
		}
		return openHandler(ticketID, openPR, openBoard)
	},
}

var commitCmd = &cobra.Command{
	Use:   "commit <message>",
	Short: "Create ticket-aware commit",
	Args:  cobra.MinimumNArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		message := strings.Join(args, " ")
		return commitHandler(message)
	},
}

var doneCmd = &cobra.Command{
	Use:   "done",
	Short: "Finalize work and create PR/MR",
	RunE: func(cmd *cobra.Command, args []string) error {
		return doneHandler()
	},
}

var configCmd = &cobra.Command{
	Use:   "config",
	Short: "Manage configuration",
}

var configShowCmd = &cobra.Command{
	Use:   "show",
	Short: "Display current configuration",
	RunE: func(cmd *cobra.Command, args []string) error {
		return configShowHandler()
	},
}

var configSetCmd = &cobra.Command{
	Use:   "set <key> <value>",
	Short: "Set a configuration value",
	Args:  cobra.ExactArgs(2),
	RunE: func(cmd *cobra.Command, args []string) error {
		return configSetHandler(args[0], args[1])
	},
}

var configValidateCmd = &cobra.Command{
	Use:   "validate",
	Short: "Validate configuration",
	RunE: func(cmd *cobra.Command, args []string) error {
		return configValHandler()
	},
}

var configPathCmd = &cobra.Command{
	Use:   "path",
	Short: "Show config path",
	RunE: func(cmd *cobra.Command, args []string) error {
		return configPathHandler()
	},
}

var (
	testJiraURL   string
	testJiraEmail string
	testJiraToken string
)

var testJiraCmd = &cobra.Command{
	Use:   "test-jira <ticket-id>",
	Short: "Test Jira API connection",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		if testJiraURL == "" || testJiraEmail == "" || testJiraToken == "" {
			return errors.New("--url, --email, and --token are required")
		}
		return testJiraHandler(args[0], testJiraURL, testJiraEmail, testJiraToken)
	},
}

func init() {
	listCmd.Flags().StringVar(&listStatus, "status", "", "Filter by status")
	listCmd.Flags().StringVar(&listProject, "project", "", "Filter by project key")
	listCmd.Flags().BoolVar(&listJSON, "json", false, "Output JSON")

	searchCmd.Flags().StringVar(&searchOpts.Assignee, "assignee", "", "Filter by assignee")
	searchCmd.Flags().StringVar(&searchOpts.Status, "status", "", "Filter by status")
	searchCmd.Flags().StringVar(&searchOpts.Project, "project", "", "Filter by project key")
	searchCmd.Flags().IntVar(&searchOpts.Limit, "limit", 10, "Maximum number of results")
	searchCmd.Flags().BoolVarP(&searchOpts.Interactive, "interactive", "i", false, "Interactive mode")

	openCmd.Flags().StringVar(&openTicket, "ticket", "", "Ticket ID")
	openCmd.Flags().BoolVar(&openPR, "pr", false, "Open PR/MR instead of ticket")
	openCmd.Flags().BoolVar(&openBoard, "board", false, "Open Jira board")

	configCmd.AddCommand(configShowCmd, configSetCmd, configValidateCmd, configPathCmd)

	testJiraCmd.Flags().StringVar(&testJiraURL, "url", "", "Jira URL")
	testJiraCmd.Flags().StringVar(&testJiraEmail, "email", "", "Jira email")
	testJiraCmd.Flags().StringVar(&testJiraToken, "token", "", "Jira API token")
}
