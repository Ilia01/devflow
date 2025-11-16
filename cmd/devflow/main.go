package main

import (
	"fmt"
	"os"

	"github.com/Ilia01/devflow/internal/app"
)

func main() {
	if err := app.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "\n%s\n", err)
		os.Exit(1)
	}
}
