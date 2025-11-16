package utils

import "strings"

const (
	colorReset = "\033[0m"
	colorBold  = "\033[1m"
	colorDim   = "\033[2m"

	colorRed         = "\033[31m"
	colorGreen       = "\033[32m"
	colorYellow      = "\033[33m"
	colorBlue        = "\033[34m"
	colorMagenta     = "\033[35m"
	colorCyan        = "\033[36m"
	colorBrightWhite = "\033[97m"
)

func Colorize(text string, codes ...string) string {
	if len(codes) == 0 {
		return text
	}
	return strings.Join(codes, "") + text + colorReset
}

func Cyan(text string) string        { return Colorize(text, colorCyan) }
func Green(text string) string       { return Colorize(text, colorGreen) }
func Yellow(text string) string      { return Colorize(text, colorYellow) }
func Red(text string) string         { return Colorize(text, colorRed) }
func Blue(text string) string        { return Colorize(text, colorBlue) }
func Magenta(text string) string     { return Colorize(text, colorMagenta) }
func BrightWhite(text string) string { return Colorize(text, colorBrightWhite) }
func Bold(text string) string        { return Colorize(text, colorBold) }
func Dim(text string) string         { return Colorize(text, colorDim) }
