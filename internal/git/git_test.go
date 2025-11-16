package git

import (
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

func TestGitClientCurrentBranch(t *testing.T) {
	dir := initTestRepo(t)
	withChdir(t, dir, func() {
		client, err := NewClient()
		if err != nil {
			t.Fatalf("NewClient failed: %v", err)
		}
		branch, err := client.CurrentBranch()
		if err != nil {
			t.Fatalf("CurrentBranch failed: %v", err)
		}
		if branch != "main" && branch != "master" {
			t.Fatalf("unexpected branch: %s", branch)
		}
	})
}

func TestGitClientCreateBranchAndCommit(t *testing.T) {
	dir := initTestRepo(t)
	withChdir(t, dir, func() {
		client, err := NewClient()
		if err != nil {
			t.Fatalf("NewClient failed: %v", err)
		}

		if err := client.CreateBranch("feat/test-branch"); err != nil {
			t.Fatalf("CreateBranch failed: %v", err)
		}

		branch, err := client.CurrentBranch()
		if err != nil {
			t.Fatalf("CurrentBranch failed: %v", err)
		}
		if branch != "feat/test-branch" {
			t.Fatalf("expected new branch, got %s", branch)
		}

		if err := os.WriteFile(filepath.Join(dir, "new.txt"), []byte("hello"), 0o644); err != nil {
			t.Fatalf("write file: %v", err)
		}

		if err := client.Commit("test commit"); err != nil {
			t.Fatalf("Commit failed: %v", err)
		}

		out := runGit(t, dir, "log", "-1", "--pretty=%B")
		if !strings.Contains(out, "test commit") {
			t.Fatalf("commit message not found: %s", out)
		}
	})
}

func TestGitStatusSummary(t *testing.T) {
	dir := initTestRepo(t)
	withChdir(t, dir, func() {
		client, err := NewClient()
		if err != nil {
			t.Fatalf("NewClient failed: %v", err)
		}

		if err := os.WriteFile(filepath.Join(dir, "file.txt"), []byte("dirty"), 0o644); err != nil {
			t.Fatalf("write file: %v", err)
		}

		summary, err := client.StatusSummary()
		if err != nil {
			t.Fatalf("StatusSummary failed: %v", err)
		}
		if !strings.Contains(summary, "file.txt") {
			t.Fatalf("expected file in status summary, got %s", summary)
		}
	})
}

func initTestRepo(t *testing.T) string {
	t.Helper()
	dir := t.TempDir()
	runGit(t, dir, "init")
	runGit(t, dir, "config", "user.email", "devflow@example.com")
	runGit(t, dir, "config", "user.name", "DevFlow Tester")
	if err := os.WriteFile(filepath.Join(dir, "README.md"), []byte("# test"), 0o644); err != nil {
		t.Fatalf("write file: %v", err)
	}
	runGit(t, dir, "add", ".")
	runGit(t, dir, "commit", "-m", "initial")
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

func withChdir(t *testing.T, dir string, fn func()) {
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
