// Package cli hosts the small bits of CLI plumbing that don't belong
// inside internal/driver or a specific subcommand's package.
//
// Currently this is just the unified error handler — the analogue of
// `crates/mdbook/src/main.rs::main`'s `utils::log_backtrace(&e);
// std::process::exit(101);` pair.
package cli

import (
	"errors"
	"fmt"
	"os"
	"strings"
)

// ExitCode is the universal exit code mdbook-go uses for any error
// path. It matches Rust mdBook's hard-coded `std::process::exit(101)`
// in src/main.rs::main. A successful run returns 0 (Go's default for
// returning from main).
//
// Note: the Rust source defines "exit 101" for every error returned
// from a subcommand; subcommand-internal exit calls (e.g. from clean
// on a missing build dir) use exit 1 in some older code paths. We
// follow the main-entry convention — every error surfaced to the
// top-level CLI uses 101.
const ExitCode = 101

// HandleError prints err to stderr in the same format Rust mdBook's
// `utils::log_backtrace` produces and then exits with ExitCode.
//
// The format is:
//
//	<err message>
//	    Caused by: <cause 1>
//	    Caused by: <cause 2>
//
// where each "Caused by:" line is one step further down err's Unwrap
// chain. We use the `errors` package's standard Unwrap walking so any
// `fmt.Errorf("...: %w", inner)` chain prints correctly.
//
// HandleError is the single replacement for the
//
//	fmt.Fprintln(os.Stderr, ...)
//	os.Exit(101)
//
// pair that previously appeared at every error site in cmd/mdbook/main.go.
// Centralising it here keeps the format consistent (matching Rust) and
// makes future additions — colour output, structured logging, etc. — a
// one-file change.
func HandleError(err error) {
	if err == nil {
		return
	}
	fmt.Fprint(os.Stderr, FormatError(err))
	os.Exit(ExitCode)
}

// FormatError returns the Rust-style error chain representation of err
// as a single string. A nil error returns the empty string. Each line
// is terminated with "\n" so concatenating multiple FormatError calls
// (or embedding one in a larger message) does not require a trailing
// newline fix-up.
//
// FormatError is exported so tests can assert the exact wire format
// without having to call HandleError (which exits and can't be tested
// directly).
func FormatError(err error) string {
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