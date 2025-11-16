package gitlab

import (
	"io"
	"net/http"
	"strings"
	"testing"
)

func TestCreateMergeRequest(t *testing.T) {
	client := NewClient("https://gitlab.example.com", "token")
	var projectLookup bool
	client.http.Transport = roundTripFunc(func(req *http.Request) *http.Response {
		switch {
		case req.Method == http.MethodGet && strings.Contains(req.URL.Path, "/projects/"):
			projectLookup = true
			return jsonResponse(http.StatusOK, `{"id":123}`)
		case req.Method == http.MethodPost && strings.Contains(req.URL.Path, "/merge_requests"):
			if !projectLookup {
				t.Fatalf("project not looked up before merge request")
			}
			return jsonResponse(http.StatusOK, `{"web_url":"https://gitlab/pr/1"}`)
		default:
			t.Fatalf("unexpected request: %s %s", req.Method, req.URL.Path)
			return nil
		}
	})

	url, err := client.CreateMergeRequest("owner/repo", "feat", "main", "title", "desc")
	if err != nil {
		t.Fatalf("CreateMergeRequest failed: %v", err)
	}
	if url != "https://gitlab/pr/1" {
		t.Fatalf("unexpected url: %s", url)
	}
}

func TestCreateMergeRequestProjectError(t *testing.T) {
	client := NewClient("https://gitlab.example.com", "token")
	client.http.Transport = roundTripFunc(func(req *http.Request) *http.Response {
		return jsonResponse(http.StatusInternalServerError, "fail")
	})
	if _, err := client.CreateMergeRequest("owner/repo", "feat", "main", "title", "desc"); err == nil {
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
