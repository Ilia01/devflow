package jira

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
	"time"

	"github.com/Ilia01/devflow/internal/config"
	"github.com/Ilia01/devflow/internal/models"
)

type Client struct {
	baseURL string
	email   string
	auth    config.AuthMethod
	http    *http.Client
}

func NewClient(baseURL, email string, auth config.AuthMethod) *Client {
	return &Client{
		baseURL: strings.TrimRight(baseURL, "/"),
		email:   email,
		auth:    auth,
		http: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

func (c *Client) apiVersion() string {
	if v := os.Getenv("JIRA_API_VERSION"); v != "" {
		return v
	}
	return "latest"
}

func (c *Client) buildURL(path string) string {
	return fmt.Sprintf("%s%s", c.baseURL, path)
}

func (c *Client) applyAuth(req *http.Request) {
	switch c.auth.Type {
	case "personal_access_token":
		req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", c.auth.Token))
	default:
		req.SetBasicAuth(c.email, c.auth.Token)
	}
}

func (c *Client) GetTicket(ticketID string) (*models.JiraTicket, error) {
	url := fmt.Sprintf("%s/rest/api/%s/issue/%s", c.baseURL, c.apiVersion(), ticketID)
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}
	c.applyAuth(req)

	var ticket models.JiraTicket
	if err := c.doJSON(req, &ticket); err != nil {
		return nil, err
	}
	return &ticket, nil
}

func (c *Client) UpdateStatus(ticketID, transitionName string) error {
	url := fmt.Sprintf("%s/rest/api/%s/issue/%s/transitions", c.baseURL, c.apiVersion(), ticketID)
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return err
	}
	c.applyAuth(req)

	var result struct {
		Transitions []struct {
			ID   string `json:"id"`
			Name string `json:"name"`
		} `json:"transitions"`
	}

	if err := c.doJSON(req, &result); err != nil {
		return err
	}

	var transitionID string
	for _, t := range result.Transitions {
		if strings.EqualFold(t.Name, transitionName) {
			transitionID = t.ID
			break
		}
	}
	if transitionID == "" {
		return fmt.Errorf("transition '%s' not found", transitionName)
	}

	payload := map[string]any{
		"transition": map[string]string{"id": transitionID},
	}
	body, err := json.Marshal(payload)
	if err != nil {
		return err
	}

	req, err = http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	c.applyAuth(req)

	if err := c.do(req, nil); err != nil {
		return err
	}
	return nil
}

func (c *Client) SearchWithJQL(jql string, limit int) ([]models.JiraTicket, error) {
	url := fmt.Sprintf("%s/rest/api/%s/search", c.baseURL, c.apiVersion())
	payload := map[string]any{
		"jql":        jql,
		"fields":     []string{"summary", "status", "assignee"},
		"maxResults": limit,
	}
	body, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}

	req, err := http.NewRequest(http.MethodPost, url, bytes.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/json")
	c.applyAuth(req)

	var response struct {
		Issues []models.JiraTicket `json:"issues"`
	}
	if err := c.doJSON(req, &response); err != nil {
		return nil, err
	}
	return response.Issues, nil
}

func (c *Client) SearchAssigned(projectKey string) ([]models.JiraTicket, error) {
	jql := fmt.Sprintf("assignee = currentUser() AND project = %s", projectKey)
	return c.SearchWithJQL(jql, 50)
}

func (c *Client) TestConnection() error {
	url := fmt.Sprintf("%s/rest/api/%s/myself", c.baseURL, c.apiVersion())
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return err
	}
	c.applyAuth(req)
	return c.do(req, nil)
}

func (c *Client) doJSON(req *http.Request, v any) error {
	return c.do(req, func(body []byte) error {
		if v == nil {
			return nil
		}
		if err := json.Unmarshal(body, v); err != nil {
			return fmt.Errorf("parse response: %w", err)
		}
		return nil
	})
}

func (c *Client) do(req *http.Request, handler func([]byte) error) error {
	resp, err := c.http.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return fmt.Errorf("jira api error (%d): %s", resp.StatusCode, string(data))
	}

	if handler != nil {
		return handler(data)
	}
	return nil
}
