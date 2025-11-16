package config

import (
	"bufio"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
)

var ErrConfigNotFound = errors.New("configuration not found")

type Settings struct {
	Jira        JiraConfig
	Git         GitConfig
	Preferences Preferences
}

type JiraConfig struct {
	URL        string
	Email      string
	ProjectKey string
	AuthMethod AuthMethod
}

type AuthMethod struct {
	Type  string
	Token string
}

type GitConfig struct {
	Provider string
	BaseURL  string
	Token    string
	Owner    string
	Repo     string
}

type Preferences struct {
	BranchPrefix      string
	DefaultTransition string
}

func Load() (*Settings, error) {
	path, err := configPath()
	if err != nil {
		return nil, err
	}

	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, ErrConfigNotFound
		}
		return nil, fmt.Errorf("read config: %w", err)
	}

	settings := &Settings{}
	if err := parseTOML(string(data), settings); err != nil {
		return nil, err
	}

	return settings, nil
}

func (s *Settings) Save() error {
	path, err := configPath()
	if err != nil {
		return err
	}

	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0o700); err != nil {
		return fmt.Errorf("create config dir: %w", err)
	}

	file, err := os.OpenFile(path, os.O_CREATE|os.O_TRUNC|os.O_WRONLY, 0o600)
	if err != nil {
		return fmt.Errorf("write config: %w", err)
	}
	defer file.Close()

	if _, err := file.WriteString(s.toTOML()); err != nil {
		return fmt.Errorf("write config: %w", err)
	}

	if err := file.Chmod(0o600); err != nil {
		return fmt.Errorf("chmod config: %w", err)
	}

	return nil
}

func ConfigDir() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("resolve home dir: %w", err)
	}
	return filepath.Join(home, ".devflow"), nil
}

func ConfigPath() (string, error) {
	dir, err := ConfigDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(dir, "config.toml"), nil
}

func configPath() (string, error) {
	return ConfigPath()
}

func parseTOML(contents string, settings *Settings) error {
	scanner := bufio.NewScanner(strings.NewReader(contents))
	section := ""
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		if strings.HasPrefix(line, "[") && strings.HasSuffix(line, "]") {
			section = strings.TrimSpace(line[1 : len(line)-1])
			continue
		}

		parts := strings.SplitN(line, "=", 2)
		if len(parts) != 2 {
			continue
		}
		key := strings.TrimSpace(parts[0])
		value := parseValue(parts[1])

		switch section {
		case "jira":
			assignJiraField(settings, key, value)
		case "jira.auth_method":
			assignJiraAuthField(settings, key, value)
		case "git":
			assignGitField(settings, key, value)
		case "preferences":
			assignPreferencesField(settings, key, value)
		}
	}

	if err := scanner.Err(); err != nil {
		return fmt.Errorf("parse config: %w", err)
	}

	return nil
}

func assignJiraField(s *Settings, key, value string) {
	switch key {
	case "url":
		s.Jira.URL = value
	case "email":
		s.Jira.Email = value
	case "project_key":
		s.Jira.ProjectKey = value
	}
}

func assignJiraAuthField(s *Settings, key, value string) {
	switch key {
	case "type":
		s.Jira.AuthMethod.Type = value
	case "token":
		s.Jira.AuthMethod.Token = value
	}
}

func assignGitField(s *Settings, key, value string) {
	switch key {
	case "provider":
		s.Git.Provider = value
	case "base_url":
		s.Git.BaseURL = value
	case "token":
		s.Git.Token = value
	case "owner":
		s.Git.Owner = value
	case "repo":
		s.Git.Repo = value
	}
}

func assignPreferencesField(s *Settings, key, value string) {
	switch key {
	case "branch_prefix":
		s.Preferences.BranchPrefix = value
	case "default_transition":
		s.Preferences.DefaultTransition = value
	}
}

func parseValue(raw string) string {
	trimmed := strings.TrimSpace(raw)
	trimmed = strings.TrimPrefix(trimmed, "\"")
	trimmed = strings.TrimSuffix(trimmed, "\"")
	return trimmed
}

func (s *Settings) toTOML() string {
	var b strings.Builder
	b.WriteString("[jira]\n")
	b.WriteString(fmt.Sprintf("url = \"%s\"\n", escape(s.Jira.URL)))
	b.WriteString(fmt.Sprintf("email = \"%s\"\n", escape(s.Jira.Email)))
	b.WriteString(fmt.Sprintf("project_key = \"%s\"\n\n", escape(s.Jira.ProjectKey)))

	b.WriteString("[jira.auth_method]\n")
	b.WriteString(fmt.Sprintf("type = \"%s\"\n", escape(s.Jira.AuthMethod.Type)))
	b.WriteString(fmt.Sprintf("token = \"%s\"\n\n", escape(s.Jira.AuthMethod.Token)))

	b.WriteString("[git]\n")
	b.WriteString(fmt.Sprintf("provider = \"%s\"\n", escape(s.Git.Provider)))
	b.WriteString(fmt.Sprintf("base_url = \"%s\"\n", escape(s.Git.BaseURL)))
	b.WriteString(fmt.Sprintf("token = \"%s\"\n", escape(s.Git.Token)))
	if s.Git.Owner != "" {
		b.WriteString(fmt.Sprintf("owner = \"%s\"\n", escape(s.Git.Owner)))
	}
	if s.Git.Repo != "" {
		b.WriteString(fmt.Sprintf("repo = \"%s\"\n", escape(s.Git.Repo)))
	}
	b.WriteString("\n")

	b.WriteString("[preferences]\n")
	b.WriteString(fmt.Sprintf("branch_prefix = \"%s\"\n", escape(s.Preferences.BranchPrefix)))
	b.WriteString(fmt.Sprintf("default_transition = \"%s\"\n", escape(s.Preferences.DefaultTransition)))
	b.WriteString("\n")

	return b.String()
}

func escape(value string) string {
	value = strings.ReplaceAll(value, "\\", "\\\\")
	value = strings.ReplaceAll(value, "\"", "\\\"")
	return value
}

func MaskToken(token string) string {
	if len(token) <= 4 {
		return strings.Repeat("*", len(token))
	}
	head := token[:min(4, len(token))]
	tail := token[max(0, len(token)-4):]
	return fmt.Sprintf("%s***%s", head, tail)
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}
