// Package completion implements the `doclens completion` subcommand:
// print a shell completion script (bash|zsh|fish|powershell) to stdout.
package completion

import (
	"errors"

	"mdbook-go/pkg/cmd/completion/bash"
	"mdbook-go/pkg/cmd/completion/fish"
	"mdbook-go/pkg/cmd/completion/powershell"
	"mdbook-go/pkg/cmd/completion/zsh"

	"github.com/spf13/cobra"
)

func NewCommand() *cobra.Command {

	cmd := &cobra.Command{
		Use:   "completion",
		Short: "Output shell completion code for the specified shell (bash, zsh, fish or powershell)",
		Long: `
Outputs doclens shell completion for the given shell (bash, fish, powershell, or zsh)
This depends on the bash-completion binary.  Example installation instructions:
# for bash users
	$ doclens completion bash > ~/.doclens-completion
	$ source ~/.doclens-completion

# for zsh users
	% doclens completion zsh > /usr/local/share/zsh/site-functions/_doclens
	% autoload -U compinit && compinit

# for fish users
	% doclens completion fish > ~/.config/fish/completions/doclens.fish

# for powershell users
	PS> doclens completion powershell | Out-String | Invoke-Expression
`,
		RunE: func(cmd *cobra.Command, args []string) error {
			err := cmd.Help()
			if err != nil {
				return err
			}
			return errors.New("Subcommand is required")
		},
	}

	cmd.AddCommand(bash.NewCommand())
	cmd.AddCommand(fish.NewCommand())
	cmd.AddCommand(powershell.NewCommand())
	cmd.AddCommand(zsh.NewCommand())

	return cmd
}
