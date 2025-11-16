package git

import (
	"bytes"
	"fmt"
	"os/exec"
	"strings"

	"github.com/Ilia01/devflow/internal/utils"
)

type Client struct {
	worktree string
}

func NewClient() (*Client, error) {
	out, err := runInDir("", "rev-parse", "--show-toplevel")
	if err != nil {
		return nil, fmt.Errorf("not in git repository: %w", err)
	}
	return &Client{worktree: strings.TrimSpace(out)}, nil
}

func (c *Client) CurrentBranch() (string, error) {
	out, err := runInDir(c.worktree, "rev-parse", "--abbrev-ref", "HEAD")
	if err != nil {
		return "", err
	}
	branch := strings.TrimSpace(out)
	if branch == "HEAD" {
		return "", fmt.Errorf("detached HEAD state")
	}
	return branch, nil
}

func (c *Client) IsClean() (bool, error) {
	out, err := runInDir(c.worktree, "status", "--porcelain")
	if err != nil {
		return false, err
	}
	return strings.TrimSpace(out) == "", nil
}

func (c *Client) StatusSummary() (string, error) {
	out, err := runInDir(c.worktree, "status", "--short")
	if err != nil {
		return "", err
	}
	trimmed := strings.TrimSpace(out)
	if trimmed == "" {
		return "  Working directory clean", nil
	}
	var lines []string
	for _, line := range strings.Split(trimmed, "\n") {
		line = strings.TrimSpace(line)
		if line == "" {
			continue
		}
		prefix := ""
		if len(line) >= 2 {
			status := strings.TrimSpace(line[:2])
			switch {
			case strings.Contains(status, "M"):
				prefix = utils.Yellow("M")
			case strings.Contains(status, "A"):
				prefix = utils.Green("A")
			case strings.Contains(status, "D"):
				prefix = utils.Red("D")
			default:
				prefix = status
			}
			path := strings.TrimSpace(line[2:])
			lines = append(lines, fmt.Sprintf("  %s %s", prefix, path))
		} else {
			lines = append(lines, "  "+line)
		}
	}
	return strings.Join(lines, "\n"), nil
}

func (c *Client) CreateBranch(branch string) error {
	_, err := runInDir(c.worktree, "checkout", "-b", branch)
	return err
}

func (c *Client) Push(branch string) error {
	_, err := runInDir(c.worktree, "push", "-u", "origin", branch)
	return err
}

func (c *Client) Commit(message string) error {
	if _, err := runInDir(c.worktree, "add", "-A"); err != nil {
		return err
	}
	cmd := exec.Command("git", "commit", "-m", message)
	cmd.Dir = c.worktree
	var stderr bytes.Buffer
	cmd.Stdout = &stderr
	cmd.Stderr = &stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("git commit failed: %s", strings.TrimSpace(stderr.String()))
	}
	return nil
}

func (c *Client) Root() string {
	return c.worktree
}

func runInDir(dir string, args ...string) (string, error) {
	cmd := exec.Command("git", args...)
	if dir != "" {
		cmd.Dir = dir
	}
	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	if err := cmd.Run(); err != nil {
		if stderr.Len() > 0 {
			return "", fmt.Errorf("%s", strings.TrimSpace(stderr.String()))
		}
		return "", err
	}
	return stdout.String(), nil
}
