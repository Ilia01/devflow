package utils

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

func Prompt(message string) (string, error) {
	fmt.Printf("%s: ", BrightWhite(message))
	reader := bufio.NewReader(os.Stdin)
	text, err := reader.ReadString('\n')
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(text), nil
}

func PromptPassword(message string) (string, error) {
	// For now we simply reuse Prompt; hiding characters would require platform specific code.
	return Prompt(message)
}

func PromptWithDefault(message, defaultValue string) (string, error) {
	fmt.Printf("%s [%s]: ", BrightWhite(message), Dim(defaultValue))
	reader := bufio.NewReader(os.Stdin)
	text, err := reader.ReadString('\n')
	if err != nil {
		return "", err
	}
	trimmed := strings.TrimSpace(text)
	if trimmed == "" {
		return defaultValue, nil
	}
	return trimmed, nil
}
