package jira

import (
	"encoding/json"
	"io"
	"net/http"
	"strings"
	"testing"

	"github.com/Ilia01/devflow/internal/config"
)

func TestGetTicket(t *testing.T) {
	client := NewClient("https://example.com", "user@example.com", config.AuthMethod{Type: "api_token", Token: "token"})
	client.http.Transport = roundTripFunc(func(req *http.Request) *http.Response {
		if req.Method != http.MethodGet || !strings.Contains(req.URL.Path, "/issue/TEST-1") {
			t.Fatalf("unexpected request: %s %s", req.Method, req.URL.Path)
		}
		body := `{"key":"TEST-1","fields":{"summary":"Test ticket","status":{"name":"To Do"}}}`
		return jsonResponse(http.StatusOK, body)
	})

	ticket, err := client.GetTicket("TEST-1")
	if err != nil {
		t.Fatalf("GetTicket failed: %v", err)
	}
	if ticket.Key != "TEST-1" {
		t.Fatalf("unexpected key: %s", ticket.Key)
	}
}

func TestUpdateStatus(t *testing.T) {
	var transitioned bool
	client := NewClient("https://example.com", "user@example.com", config.AuthMethod{Type: "personal_access_token", Token: "token"})
	client.http.Transport = roundTripFunc(func(req *http.Request) *http.Response {
		switch {
		case req.Method == http.MethodGet && strings.HasSuffix(req.URL.Path, "/transitions"):
			body := `{"transitions":[{"id":"1","name":"In Progress"}]}`
			return jsonResponse(http.StatusOK, body)
		case req.Method == http.MethodPost && strings.HasSuffix(req.URL.Path, "/transitions"):
			transitioned = true
			return jsonResponse(http.StatusNoContent, "")
		default:
			t.Fatalf("unexpected request: %s %s", req.Method, req.URL.Path)
			return nil
		}
	})

	if err := client.UpdateStatus("TEST-1", "In Progress"); err != nil {
		t.Fatalf("UpdateStatus failed: %v", err)
	}
	if !transitioned {
		t.Fatalf("transition endpoint not invoked")
	}
}

func TestSearchWithJQL(t *testing.T) {
	client := NewClient("https://example.com", "user@example.com", config.AuthMethod{Type: "api_token", Token: "token"})
	client.http.Transport = roundTripFunc(func(req *http.Request) *http.Response {
		if req.Method != http.MethodPost || !strings.HasSuffix(req.URL.Path, "/search") {
			t.Fatalf("unexpected request: %s %s", req.Method, req.URL.Path)
		}
		var payload map[string]any
		if err := json.NewDecoder(req.Body).Decode(&payload); err != nil {
			t.Fatalf("decode payload: %v", err)
		}
		if payload["jql"] != "project = TEST" {
			t.Fatalf("unexpected jql: %v", payload["jql"])
		}
		body := `{"issues":[{"key":"TEST-2","fields":{"summary":"Another","status":{"name":"Done"}}}]}`
		return jsonResponse(http.StatusOK, body)
	})

	tickets, err := client.SearchWithJQL("project = TEST", 5)
	if err != nil {
		t.Fatalf("SearchWithJQL failed: %v", err)
	}
	if len(tickets) != 1 || tickets[0].Key != "TEST-2" {
		t.Fatalf("unexpected results: %#v", tickets)
	}
}

func TestTestConnection(t *testing.T) {
	client := NewClient("https://example.com", "user@example.com", config.AuthMethod{Type: "api_token", Token: "token"})
	client.http.Transport = roundTripFunc(func(req *http.Request) *http.Response {
		return jsonResponse(http.StatusOK, "")
	})
	if err := client.TestConnection(); err != nil {
		t.Fatalf("TestConnection failed: %v", err)
	}
}

type roundTripFunc func(*http.Request) *http.Response

func (f roundTripFunc) RoundTrip(req *http.Request) (*http.Response, error) {
	return f(req), nil
}

func jsonResponse(status int, body string) *http.Response {
	return &http.Response{
		StatusCode: status,
		Body:       io.NopCloser(strings.NewReader(body)),
		Header:     make(http.Header),
	}
}
