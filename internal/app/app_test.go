package app

import (
	"bytes"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"

	"github.com/Ilia01/devflow/internal/config"
	"github.com/Ilia01/devflow/internal/models"
)

func TestStartCommandInvokesHandler(t *testing.T) {
	restoreStart := swapStartHandler(func(ticket string) error {
		if ticket != "WAB-123" {
			t.Fatalf("unexpected ticket: %s", ticket)
		}
		return nil
	})
	defer restoreStart()

	if err := executeCLI("start", "WAB-123"); err != nil {
		t.Fatalf("start command failed: %v", err)
	}
}

func TestOpenCommandFlags(t *testing.T) {
	restoreOpen := swapOpenHandler(func(ticket string, pr, board bool) error {
		if ticket != "WAB-9" {
			t.Fatalf("unexpected ticket: %s", ticket)
		}
		if !pr || board {
			t.Fatalf("flags not passed correctly")
		}
		return nil
	})
	defer restoreOpen()

	resetOpenFlags()
	if err := executeCLI("open", "--ticket", "WAB-9", "--pr"); err != nil {
		t.Fatalf("open command failed: %v", err)
	}
}

func TestConfigSetCommand(t *testing.T) {
	called := false
	restoreSet := swapConfigSetHandler(func(key, value string) error {
		called = true
		if key != "jira.url" || value != "https://jira" {
			t.Fatalf("unexpected args %s %s", key, value)
		}
		return nil
	})
	defer restoreSet()

	if err := executeCLI("config", "set", "jira.url", "https://jira"); err != nil {
		t.Fatalf("config set failed: %v", err)
	}
	if !called {
		t.Fatalf("handler not called")
	}
}

func TestTestJiraCommandRequiresFlags(t *testing.T) {
	restore := swapTestJiraHandler(func(ticket, url, email, token string) error {
		if ticket != "WAB-1" || url != "https://jira" || email != "user" || token != "tok" {
			t.Fatalf("unexpected args")
		}
		return nil
	})
	defer restore()

	if err := executeCLI("test-jira", "--url", "https://jira", "--email", "user", "--token", "tok", "WAB-1"); err != nil {
		t.Fatalf("test-jira failed: %v", err)
	}
}

func TestEndToEndWorkflow(t *testing.T) {
	homeDir := t.TempDir()
	t.Setenv("HOME", homeDir)

	repoDir := initRepoWithRemote(t)
	fakeJira := newFakeJiraClient()
	restoreJira := swapJiraFactory(func(url, email string, auth config.AuthMethod) jiraService {
		return fakeJira
	})
	defer restoreJira()

	fakeGitLab := &fakeGitLabClient{}
	restoreGitLab := swapGitLabFactory(func(baseURL, token string) gitLabService {
		return fakeGitLab
	})
	defer restoreGitLab()

	settings := &config.Settings{
		Jira: config.JiraConfig{
			URL:        "https://jira.example.com",
			Email:      "dev@example.com",
			ProjectKey: "TEST",
			AuthMethod: config.AuthMethod{Type: "api_token", Token: "token"},
		},
		Git: config.GitConfig{
			Provider: "gitlab",
			BaseURL:  "https://gitlab.example.com",
			Token:    "git-token",
		},
		Preferences: config.Preferences{
			BranchPrefix:      "feat",
			DefaultTransition: "In Progress",
		},
	}
	if err := settings.Save(); err != nil {
		t.Fatalf("failed to save settings: %v", err)
	}

	withDir(t, repoDir, func() {
		if err := executeCLI("start", "TEST-1"); err != nil {
			t.Fatalf("start command failed: %v", err)
		}

		if err := os.WriteFile("feature.txt", []byte("hello world"), 0o644); err != nil {
			t.Fatalf("write file: %v", err)
		}

		if err := executeCLI("commit", "Add feature"); err != nil {
			t.Fatalf("commit failed: %v", err)
		}

		if err := executeCLI("done"); err != nil {
			t.Fatalf("done failed: %v", err)
		}
	})

	if fakeGitLab.mergeCount == 0 {
		t.Fatalf("expected merge request to be created")
	}

	if len(fakeJira.transitions) < 2 {
		t.Fatalf("expected transitions recorded, got %v", fakeJira.transitions)
	}
	if fakeJira.transitions[0] != "In Progress" || fakeJira.transitions[len(fakeJira.transitions)-1] != "In Review" {
		t.Fatalf("unexpected transition sequence: %v", fakeJira.transitions)
	}

	remoteRefs := runGit(t, repoDir, "ls-remote", "origin", "refs/heads/feat/TEST-1/implement_widget")
	if strings.TrimSpace(remoteRefs) == "" {
		t.Fatalf("expected branch pushed to remote")
	}

	log := runGit(t, repoDir, "log", "-1", "--pretty=%B")
	if !strings.Contains(log, "TEST-1") {
		t.Fatalf("ticket id missing from commit message: %s", log)
	}
}

func executeCLI(args ...string) error {
	rootCmd.SetArgs(args)
	rootCmd.SetOut(bytes.NewBuffer(nil))
	rootCmd.SetErr(bytes.NewBuffer(nil))
	return rootCmd.Execute()
}

func resetOpenFlags() {
	openTicket = ""
	openPR = false
	openBoard = false
}

func swapStartHandler(fn func(string) error) func() {
	orig := startHandler
	startHandler = fn
	return func() { startHandler = orig }
}

func swapOpenHandler(fn func(string, bool, bool) error) func() {
	orig := openHandler
	openHandler = fn
	return func() { openHandler = orig }
}

func swapConfigSetHandler(fn func(string, string) error) func() {
	orig := configSetHandler
	configSetHandler = fn
	return func() { configSetHandler = orig }
}

func swapTestJiraHandler(fn func(string, string, string, string) error) func() {
	orig := testJiraHandler
	testJiraHandler = fn
	return func() { testJiraHandler = orig }
}

func initRepoWithRemote(t *testing.T) string {
	dir := t.TempDir()
	runGit(t, dir, "init", "-b", "main")
	runGit(t, dir, "config", "user.email", "devflow@example.com")
	runGit(t, dir, "config", "user.name", "DevFlow Tester")
	if err := os.WriteFile(filepath.Join(dir, "README.md"), []byte("# test"), 0o644); err != nil {
		t.Fatalf("write file: %v", err)
	}
	runGit(t, dir, "add", ".")
	runGit(t, dir, "commit", "-m", "initial")

	remoteParent := t.TempDir()
	remotePath := filepath.Join(remoteParent, "remote.git")
	cmd := exec.Command("git", "init", "--bare", remotePath)
	if out, err := cmd.CombinedOutput(); err != nil {
		t.Fatalf("init bare repo: %v (%s)", err, string(out))
	}
	runGit(t, dir, "remote", "add", "origin", remotePath)
	return dir
}

func runGit(t *testing.T, dir string, args ...string) string {
	t.Helper()
	cmd := exec.Command("git", args...)
	cmd.Dir = dir
	out, err := cmd.CombinedOutput()
	if err != nil {
		t.Fatalf("git %v failed: %v (%s)", args, err, string(out))
	}
	return string(out)
}

func withDir(t *testing.T, dir string, fn func()) {
	t.Helper()
	orig, err := os.Getwd()
	if err != nil {
		t.Fatalf("getwd: %v", err)
	}
	if err := os.Chdir(dir); err != nil {
		t.Fatalf("chdir: %v", err)
	}
	t.Cleanup(func() {
		_ = os.Chdir(orig)
	})
	fn()
}

type fakeJiraClient struct {
	ticket      *models.JiraTicket
	transitions []string
}

func newFakeJiraClient() *fakeJiraClient {
	return &fakeJiraClient{
		ticket: &models.JiraTicket{
			Key: "TEST-1",
			Fields: models.TicketFields{
				Summary: "Implement widget",
				Status:  models.TicketStatus{Name: "To Do"},
			},
		},
	}
}

func (f *fakeJiraClient) GetTicket(ticketID string) (*models.JiraTicket, error) {
	return f.ticket, nil
}

func (f *fakeJiraClient) UpdateStatus(ticketID, status string) error {
	f.transitions = append(f.transitions, status)
	return nil
}

func (f *fakeJiraClient) SearchWithJQL(string, int) ([]models.JiraTicket, error) {
	return nil, nil
}

func (f *fakeJiraClient) TestConnection() error {
	return nil
}

type fakeGitLabClient struct {
	mergeCount int
}

func (f *fakeGitLabClient) CreateMergeRequest(projectPath, sourceBranch, targetBranch, title, description string) (string, error) {
	f.mergeCount++
	return "https://gitlab.example.com/mr/1", nil
}

func swapJiraFactory(fn func(string, string, config.AuthMethod) jiraService) func() {
	orig := jiraFactory
	jiraFactory = fn
	return func() { jiraFactory = orig }
}

func swapGitLabFactory(fn func(string, string) gitLabService) func() {
	orig := gitLabFactory
	gitLabFactory = fn
	return func() { gitLabFactory = orig }
}
