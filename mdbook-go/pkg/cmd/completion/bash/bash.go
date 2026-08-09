// Package bash implements the `doclens completion bash` subcommand:
// generate the bash completion script.
package bash

import (
	"os"

	"github.com/spf13/cobra"
)

func NewCommand() *cobra.Command {
	cmd := &cobra.Command{
		Args:  cobra.NoArgs,
		Use:   "bash",
		Short: "Output shell completions for bash",
		RunE: func(cmd *cobra.Command, args []string) error {
			return cmd.Root().GenBashCompletion(os.Stdout)
		},
	}
	return cmd
}
