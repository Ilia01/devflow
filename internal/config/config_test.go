package config

import (
	"errors"
	"os"
	"testing"
)

func TestSaveAndLoadRoundTrip(t *testing.T) {
	tmpDir := t.TempDir()
	t.Setenv("HOME", tmpDir)

	settings := &Settings{
		Jira: JiraConfig{
			URL:        "https://jira.example.com",
			Email:      "user@example.com",
			ProjectKey: "PROJ",
			AuthMethod: AuthMethod{Type: "api_token", Token: "secret"},
		},
		Git: GitConfig{
			Provider: "github",
			BaseURL:  "https://api.github.com",
			Token:    "gh-token",
			Owner:    "owner",
			Repo:     "repo",
		},
		Preferences: Preferences{
			BranchPrefix:      "feat",
			DefaultTransition: "In Progress",
		},
	}

	if err := settings.Save(); err != nil {
		t.Fatalf("save failed: %v", err)
	}

	cfgPath, err := ConfigPath()
	if err != nil {
		t.Fatalf("config path: %v", err)
	}
	if _, err := os.Stat(cfgPath); err != nil {
		t.Fatalf("config file missing: %v", err)
	}

	loaded, err := Load()
	if err != nil {
		t.Fatalf("load failed: %v", err)
	}

	if loaded.Jira.URL != settings.Jira.URL {
		t.Fatalf("jira url mismatch: got %s want %s", loaded.Jira.URL, settings.Jira.URL)
	}
	if loaded.Git.Provider != settings.Git.Provider {
		t.Fatalf("git provider mismatch: got %s", loaded.Git.Provider)
	}
	if loaded.Preferences.BranchPrefix != settings.Preferences.BranchPrefix {
		t.Fatalf("branch prefix mismatch: got %s", loaded.Preferences.BranchPrefix)
	}
}

func TestLoadMissingConfig(t *testing.T) {
	tmpDir := t.TempDir()
	t.Setenv("HOME", tmpDir)

	_, err := Load()
	if err == nil {
		t.Fatal("expected error for missing config")
	}
	if !errors.Is(err, ErrConfigNotFound) {
		t.Fatalf("expected ErrConfigNotFound got %v", err)
	}
}

func TestMaskToken(t *testing.T) {
	token := "abcdefgh"
	masked := MaskToken(token)
	if masked != "abcd***efgh" {
		t.Fatalf("unexpected mask: %s", masked)
	}

	short := "abc"
	if MaskToken(short) != "***" {
		t.Fatalf("short mask unexpected: %s", MaskToken(short))
	}
}
