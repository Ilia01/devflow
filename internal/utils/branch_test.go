package utils

import "testing"

func TestFormatBranchName(t *testing.T) {
	tests := []struct {
		name     string
		prefix   string
		ticketID string
		summary  string
		want     string
	}{
		{"basic", "feat", "WAB-1234", "Add user authentication", "feat/WAB-1234/add_user_authentication"},
		{"special chars", "fix", "PROJ-999", "Fix bug: login doesn't work!", "fix/PROJ-999/fix_bug_login_doesnt_work"},
		{"numbers", "feat", "ABC-42", "Update Node.js to v20", "feat/ABC-42/update_node_js_to_v20"},
		{"empty summary", "test", "TICKET-1", "", "test/TICKET-1"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := FormatBranchName(tt.prefix, tt.ticketID, tt.summary)
			if got != tt.want {
				t.Fatalf("FormatBranchName() = %s, want %s", got, tt.want)
			}
		})
	}
}

func TestExtractTicketID(t *testing.T) {
	id, err := ExtractTicketID("feat/WAB-3848/implement_attempts_doc_logic")
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if id != "WAB-3848" {
		t.Fatalf("wrong ticket id: %s", id)
	}

	if _, err := ExtractTicketID("main"); err == nil {
		t.Fatalf("expected error for branch without ticket")
	}
}
