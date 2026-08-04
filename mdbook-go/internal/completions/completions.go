// Package completions generates shell completion scripts for the mdbook-go
// CLI. It is the Go analogue of `clap_complete::generate(shell, ...)` in
// src/main.rs::main and matches the four shells clap_complete supports out
// of the box: bash, zsh, fish, and PowerShell.
//
// The generated scripts are intentionally static and self-contained — they
// hard-code the subcommand and flag list rather than reading it back from
// the binary, so they remain valid even when mdbook-go is invoked through
// a stripped-down PATH. The user is expected to source or install the
// output once per shell, per version:
//
//	# bash
//	mdbook-go completions bash > /etc/bash_completion.d/mdbook-go
//	# or, for a single user:
//	mdbook-go completions bash > ~/.local/share/bash-completion/completions/mdbook-go
//
//	# zsh (oh-my-zsh / completions directory)
//	mdbook-go completions zsh > "${fpath[1]}/_mdbook-go"
//
//	# fish
//	mdbook-go completions fish > ~/.config/fish/completions/mdbook-go.fish
//
//	# PowerShell (current user)
//	mdbook-go completions powershell | Out-String | Invoke-Expression
package completions

import (
	"fmt"
	"io"
	"strings"
)

// Shell is the target shell for a completion script.
type Shell string

const (
	Bash       Shell = "bash"
	Zsh        Shell = "zsh"
	Fish       Shell = "fish"
	PowerShell Shell = "powershell"
)

// ParseShell accepts the canonical shell name (case-insensitive) and
// returns the matching Shell value. The empty string and any unknown
// value produce an error so the CLI can fail loudly rather than emit a
// garbage script.
func ParseShell(s string) (Shell, error) {
	switch strings.ToLower(strings.TrimSpace(s)) {
	case "":
		return "", fmt.Errorf("missing shell name (expected bash|zsh|fish|powershell)")
	case "bash":
		return Bash, nil
	case "zsh":
		return Zsh, nil
	case "fish":
		return Fish, nil
	case "powershell", "pwsh":
		return PowerShell, nil
	}
	return "", fmt.Errorf("unsupported shell %q (expected bash|zsh|fish|powershell)", s)
}

// commandSpec describes a single subcommand and its flags. The flag list
// is the long form (with leading "--") and is reused across all four
// shells. Short flags are listed separately so bash / zsh can attach them
// to the same -s/--long pair.
type commandSpec struct {
	name  string
	short string   // one-line help (used by zsh's _describe)
	flags []string // long names including the leading "--"
	shortFlags string // short names without dash, e.g. "d" or empty
}

// spec is the single source of truth for the CLI surface. It is consulted
// by every script generator so that adding a subcommand or flag only
// requires editing this table.
var spec = []commandSpec{
	{
		name:  "init",
		short: "create a new book skeleton",
		flags: []string{"--dir", "--theme", "--force", "--title", "--ignore"},
	},
	{
		name:  "build",
		short: "build a book",
		flags: []string{"--dir", "--dest-dir", "--open"},
		shortFlags: "d",
	},
	{
		name:  "clean",
		short: "remove the build directory",
		flags: []string{"--dir", "--dest-dir"},
		shortFlags: "d",
	},
	{
		name:  "test",
		short: "run rustdoc --test on chapters",
		flags: []string{"--dir", "--chapter", "--library-path"},
	},
	{
		name:  "watch",
		short: "rebuild on file changes",
		flags: []string{"--dir", "--dest-dir", "--open", "--watcher"},
		shortFlags: "d",
	},
	{
		name:  "serve",
		short: "serve the book + live reload",
		flags: []string{"--dir", "--dest-dir", "--hostname", "--port", "--open"},
		shortFlags: "d",
	},
	{
		name:  "version",
		short: "show version",
		flags: nil,
	},
	{
		name:  "completions",
		short: "generate shell completions",
		flags: []string{"--shell"},
	},
}

// Generate writes the completion script for shell to w.
func Generate(w io.Writer, shell Shell) error {
	switch shell {
	case Bash:
		return writeBash(w)
	case Zsh:
		return writeZsh(w)
	case Fish:
		return writeFish(w)
	case PowerShell:
		return writePowerShell(w)
	}
	return fmt.Errorf("unsupported shell %q", shell)
}

// subcommandNames returns the list of subcommand names. Used by the
// first-word completion tables in every shell.
func subcommandNames() []string {
	out := make([]string, 0, len(spec))
	for _, c := range spec {
		out = append(out, c.name)
	}
	return out
}

// writeBash emits a bash completion script compatible with bash-completion
// ≥ 2.0 (the version shipped by every maintained Linux distro and macOS
// via Homebrew). The script uses `complete -F` so it is loaded by sourcing
// the file rather than via a dynamic loader.
func writeBash(w io.Writer) error {
	var b strings.Builder
	b.WriteString(`# bash completion for mdbook-go
# Install: source this file or drop it into $BASH_COMPLETION_USER_DIR/completions/

_mdbook_go() {
    local cur prev cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmds="`)
	b.WriteString(strings.Join(subcommandNames(), " "))
	b.WriteString(`"

    if [[ ${COMP_CWORD} -eq 1 ]] ; then
        COMPREPLY=( $(compgen -W "${cmds}" -- "${cur}") )
        return 0
    fi

    local subcmd="${COMP_WORDS[1]}"
    case "${subcmd}" in
`)
	for _, c := range spec {
		if len(c.flags) == 0 {
			continue
		}
		b.WriteString("        ")
		b.WriteString(c.name)
		b.WriteString(")\n")
		b.WriteString(`            local opts="`)
		b.WriteString(strings.Join(c.flags, " "))
		b.WriteString(`"
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
`)
	}
	b.WriteString(`    esac
}

complete -F _mdbook_go mdbook-go
`)
	_, err := io.WriteString(w, b.String())
	return err
}

// writeZsh emits a zsh completion script that follows the
// `#compdef mdbook-go` convention so it can be dropped straight into any
// directory on $fpath.
func writeZsh(w io.Writer) error {
	var b strings.Builder
	b.WriteString(`#compdef mdbook-go
# zsh completion for mdbook-go. Install under any directory on $fpath
# (typically ~/.zsh/site-functions/_mdbook-go or
# $XDG_DATA_HOME/zsh/site-functions/_mdbook-go).

_mdbook_go() {
    local -a commands
    commands=(
`)
	for _, c := range spec {
		fmt.Fprintf(&b, "        '%s:%s'\n", c.name, c.short)
	}
	b.WriteString(`    )

    if (( CURRENT == 2 )); then
        _describe 'command' commands
        return 0
    fi

    local cmd="${words[2]}"
    case "${cmd}" in
`)
	for _, c := range spec {
		if len(c.flags) == 0 {
			continue
		}
		fmt.Fprintf(&b, "        %s)\n", c.name)
		b.WriteString(`            local -a opts
            opts=(`)
		for _, f := range c.flags {
			fmt.Fprintf(&b, "\n                '%s'", f)
		}
		b.WriteString("\n            )\n            _describe 'option' opts\n            return 0\n            ;;\n")
	}
	b.WriteString(`    esac
}

_mdbook_go "$@"
`)
	_, err := io.WriteString(w, b.String())
	return err
}

// writeFish emits a fish completion script using the `complete -c` form.
// The script is short enough that it can be eval'd from $fish_config
// without on-disk installation.
func writeFish(w io.Writer) error {
	var b strings.Builder
	b.WriteString(`# fish completion for mdbook-go
# Install: copy to ~/.config/fish/completions/mdbook-go.fish

function __mdbook_go_subcommands
    echo "`)
	b.WriteString(strings.Join(subcommandNames(), "\n    "))
	b.WriteString(`"
end

for sub in (__mdbook_go_subcommands)
    complete -c mdbook-go -n "__fish_use_subcommand" -a "$sub" -f
end

`)
	for _, c := range spec {
		if len(c.flags) == 0 {
			continue
		}
		fmt.Fprintf(&b, "complete -c mdbook-go -n \"__fish_seen_subcommand_from %s\" -l ", c.name)
		// fish's -l flag accepts the long name without the leading "--".
		for i, f := range c.flags {
			if !strings.HasPrefix(f, "--") {
				continue
			}
			if i > 0 {
				b.WriteString(" -l ")
			}
			b.WriteString(strings.TrimPrefix(f, "--"))
		}
		b.WriteString("\n")
	}
	_, err := io.WriteString(w, b.String())
	return err
}

// writePowerShell emits a PowerShell completion script compatible with
// PSReadLine. The Register-ArgumentCompleter cmdlet is the standard
// PowerShell 5.1+ / Core mechanism.
func writePowerShell(w io.Writer) error {
	var b strings.Builder
	b.WriteString(`# PowerShell completion for mdbook-go.
# Install: run `)
	b.WriteString("`mdbook-go completions powershell | Out-String | Invoke-Expression`")
	b.WriteString(` in your $PROFILE.

using namespace System.Management.Automation
using namespace System.Collections.ObjectModel

Register-ArgumentCompleter -Native -CommandName 'mdbook-go' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $subcommands = @('`)
	b.WriteString(strings.Join(subcommandNames(), "', '"))
	b.WriteString(`')

    $token = $commandAst.ToString()
    $tokens = $token -split '\s+'

    if ($tokens.Count -le 2) {
        $subcommands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_)
        }
        return
    }

    $sub = $tokens[1]
    $flagMap = @{
`)
	for _, c := range spec {
		if len(c.flags) == 0 {
			continue
		}
		fmt.Fprintf(&b, "        '%s' = @('", c.name)
		b.WriteString(strings.Join(c.flags, "', '"))
		b.WriteString("')\n")
	}
	b.WriteString(`    }

    if ($flagMap.ContainsKey($sub)) {
        $flagMap[$sub] | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
            [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterName', $_)
        }
    }
}
`)
	_, err := io.WriteString(w, b.String())
	return err
}