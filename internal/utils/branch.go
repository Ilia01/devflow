package utils

import (
	"fmt"
	"strings"
	"unicode"
)

func FormatBranchName(prefix, ticketID, summary string) string {
	lowered := strings.ToLower(summary)
	words := strings.FieldsFunc(lowered, func(r rune) bool {
		switch r {
		case ' ', ':', '!', '?', ',', ';', '.':
			return true
		}
		return false
	})

	cleaned := make([]string, 0, len(words))
	for _, word := range words {
		var builder strings.Builder
		for _, r := range word {
			if unicode.IsLetter(r) || unicode.IsDigit(r) {
				builder.WriteRune(r)
			}
		}
		candidate := builder.String()
		if len(candidate) > 1 {
			cleaned = append(cleaned, candidate)
		}
		if len(cleaned) == 5 {
			break
		}
	}

	if len(cleaned) == 0 {
		return fmt.Sprintf("%s/%s", prefix, ticketID)
	}

	return fmt.Sprintf("%s/%s/%s", prefix, ticketID, strings.Join(cleaned, "_"))
}

func ExtractTicketID(branch string) (string, error) {
	parts := strings.Split(branch, "/")
	if len(parts) < 2 {
		return "", fmt.Errorf("branch '%s' does not contain a ticket id", branch)
	}

	ticketPart := parts[1]
	if !strings.Contains(ticketPart, "-") {
		return "", fmt.Errorf("branch '%s' does not contain a ticket id", branch)
	}

	segments := strings.Split(ticketPart, "-")
	if len(segments) < 2 {
		return "", fmt.Errorf("branch '%s' does not contain a ticket id", branch)
	}

	ticketID := strings.Join(segments[:2], "-")
	if ticketID == "" {
		return "", fmt.Errorf("branch '%s' does not contain a ticket id", branch)
	}

	return ticketID, nil
}
