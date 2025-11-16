package gitlab

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

type Client struct {
	baseURL string
	token   string
	http    *http.Client
}

func NewClient(baseURL, token string) *Client {
	return &Client{
		baseURL: strings.TrimRight(baseURL, "/"),
		token:   token,
		http:    &http.Client{Timeout: 30 * time.Second},
	}
}

func (c *Client) CreateMergeRequest(projectPath, sourceBranch, targetBranch, title, description string) (string, error) {
	projectID, err := c.getProjectID(projectPath)
	if err != nil {
		return "", err
	}

	payload := map[string]any{
		"source_branch":        sourceBranch,
		"target_branch":        targetBranch,
		"title":                title,
		"description":          description,
		"remove_source_branch": true,
	}
	body, err := json.Marshal(payload)
	if err != nil {
		return "", err
	}

	req, err := http.NewRequest(http.MethodPost, fmt.Sprintf("%s/api/v4/projects/%d/merge_requests", c.baseURL, projectID), bytes.NewReader(body))
	if err != nil {
		return "", err
	}
	req.Header.Set("PRIVATE-TOKEN", c.token)
	req.Header.Set("Content-Type", "application/json")

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
		return "", fmt.Errorf("gitlab api error (%d): %s", resp.StatusCode, string(data))
	}

	var result struct {
		WebURL string `json:"web_url"`
	}
	if err := json.Unmarshal(data, &result); err != nil {
		return "", err
	}
	return result.WebURL, nil
}

func (c *Client) getProjectID(projectPath string) (int64, error) {
	encoded := url.PathEscape(projectPath)
	req, err := http.NewRequest(http.MethodGet, fmt.Sprintf("%s/api/v4/projects/%s", c.baseURL, encoded), nil)
	if err != nil {
		return 0, err
	}
	req.Header.Set("PRIVATE-TOKEN", c.token)

	resp, err := c.http.Do(req)
	if err != nil {
		return 0, err
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return 0, err
	}

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return 0, fmt.Errorf("gitlab api error (%d): %s", resp.StatusCode, string(data))
	}

	var result struct {
		ID int64 `json:"id"`
	}
	if err := json.Unmarshal(data, &result); err != nil {
		return 0, err
	}
	return result.ID, nil
}
