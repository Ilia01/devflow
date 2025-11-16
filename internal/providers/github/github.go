package github

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type Client struct {
	owner   string
	repo    string
	token   string
	http    *http.Client
	baseURL string
}

func NewClient(owner, repo, token string) *Client {
	return &Client{
		owner:   owner,
		repo:    repo,
		token:   token,
		http:    &http.Client{Timeout: 30 * time.Second},
		baseURL: "https://api.github.com",
	}
}

func (c *Client) CreatePullRequest(sourceBranch, targetBranch, title, description string) (string, error) {
	payload := map[string]string{
		"title": title,
		"body":  description,
		"head":  sourceBranch,
		"base":  targetBranch,
	}
	body, err := json.Marshal(payload)
	if err != nil {
		return "", err
	}

	url := fmt.Sprintf("%s/repos/%s/%s/pulls", c.baseURL, c.owner, c.repo)
	req, err := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return "", err
	}
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", c.token))
	req.Header.Set("Accept", "application/vnd.github+json")
	req.Header.Set("User-Agent", "devflow-cli")

	resp, err := c.http.Do(req)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return "", fmt.Errorf("github api error (%d): %s", resp.StatusCode, string(data))
	}

	var result struct {
		HTMLURL string `json:"html_url"`
	}
	if err := json.Unmarshal(data, &result); err != nil {
		return "", err
	}
	return result.HTMLURL, nil
}
