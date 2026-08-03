// mdbook-go is the Go-language port of mdBook. Only the build and init
// subcommands are wired up in M1; further subcommands arrive in later
// milestones.
package main

import (
	"flag"
	"fmt"
	"os"

	"mdbook-go/internal/driver"
)

func main() {
	if len(os.Args) < 2 {
		usage()
		os.Exit(101)
	}
	cmd := os.Args[1]
	args := os.Args[2:]

	switch cmd {
	case "init":
		fs := flag.NewFlagSet("init", flag.ExitOnError)
		dir := fs.String("dir", ".", "book root")
		copyTheme := fs.Bool("theme", false, "copy default theme")
		_ = fs.Parse(args)
		if err := driver.Init(*dir, *copyTheme); err != nil {
			fmt.Fprintf(os.Stderr, "init: %v\n", err)
			os.Exit(101)
		}
	case "build":
		fs := flag.NewFlagSet("build", flag.ExitOnError)
		dir := fs.String("dir", ".", "book root")
		dest := fs.String("dest-dir", "", "output directory (overrides book.toml)")
		_ = fs.Parse(args)
		if err := runBuild(*dir, *dest); err != nil {
			fmt.Fprintf(os.Stderr, "build: %v\n", err)
			os.Exit(101)
		}
	case "version", "--version", "-v":
		fmt.Println("mdbook-go 0.1.0 (M2 closed; M3 in flight)")
	default:
		usage()
		os.Exit(101)
	}
}

func runBuild(dir, dest string) error {
	m, err := driver.Load(dir)
	if err != nil {
		return err
	}
	if dest != "" {
		m.Config.Build.BuildDir = dest
	}
	return m.Build()
}

func usage() {
	fmt.Fprintln(os.Stderr, "mdbook-go <command> [args]")
	fmt.Fprintln(os.Stderr, "  init   [-dir DIR] [-theme]   create a new book skeleton")
	fmt.Fprintln(os.Stderr, "  build  [-dir DIR] [-dest-dir DIR]   build a book")
	fmt.Fprintln(os.Stderr, "  version                       show version")
}
