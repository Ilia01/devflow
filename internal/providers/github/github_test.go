package github

import (
	"io"
	"net/http"
	"strings"
	"testing"
)

func TestCreatePullRequest(t *testing.T) {
	client := NewClient("owner", "repo", "token")
	client.http.Transport = roundTripFunc(func(req *http.Request) *http.Response {
		if req.Method != http.MethodPost {
			t.Fatalf("expected POST got %s", req.Method)
		}
		return jsonResponse(http.StatusOK, `{"html_url":"https://example.com/pr/1"}`)
	})

	url, err := client.CreatePullRequest("feat", "main", "title", "desc")
	if err != nil {
		t.Fatalf("CreatePullRequest failed: %v", err)
	}
	if url != "https://example.com/pr/1" {
		t.Fatalf("unexpected url: %s", url)
	}
}

func TestCreatePullRequestError(t *testing.T) {
	client := NewClient("owner", "repo", "token")
	client.http.Transport = roundTripFunc(func(req *http.Request) *http.Response {
		return jsonResponse(http.StatusInternalServerError, "fail")
	})

	if _, err := client.CreatePullRequest("feat", "main", "title", "desc"); err == nil {
		t.Fatalf("expected error")
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
