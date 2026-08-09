// Command doclens is the mdbook-go CLI entry point. Each subcommand returns
// an error; main is the single place that formats it and exits, mirroring
// Rust mdBook's src/main.rs::main, which calls utils::log_backtrace(&e)
// and std::process::exit(101).
package main

import (
	"errors"
	"fmt"
	"os"
	"strings"

	"mdbook-go/pkg/cmd"
)

// exitCode is the universal exit code for any error path, matching Rust
// mdBook's hard-coded std::process::exit(101) in src/main.rs::main. A
// successful run returns 0 (Go's default for returning from main).
const exitCode = 101

func main() {
	if err := cmd.New().Execute(); err != nil {
		fmt.Fprint(os.Stderr, formatError(err))
		os.Exit(exitCode)
	}
}

// formatError returns the Rust-style error chain representation of err as
// a single string:
//
//	<err message>
//	    Caused by: <cause 1>
//	    Caused by: <cause 2>
//
// where each "Caused by:" line is one step further down err's Unwrap chain.
// The standard errors.Unwrap walk means any fmt.Errorf("...: %w", inner)
// chain prints correctly. Every line is terminated with "\n"; a nil error
// returns the empty string.
func formatError(err error) string {
	if err == nil {
		return ""
	}
	var b strings.Builder
	b.WriteString(err.Error())
	b.WriteString("\n")
	for cur := errors.Unwrap(err); cur != nil; cur = errors.Unwrap(cur) {
		b.WriteString("\tCaused by: ")
		b.WriteString(cur.Error())
		b.WriteString("\n")
	}
	return b.String()
}
