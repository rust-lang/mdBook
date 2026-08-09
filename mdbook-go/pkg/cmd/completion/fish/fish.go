// Package fish implements the `doclens completion fish` subcommand:
// generate the fish completion script.
package fish

import (
	"os"

	"github.com/spf13/cobra"
)

func NewCommand() *cobra.Command {
	cmd := &cobra.Command{
		Args:  cobra.NoArgs,
		Use:   "fish",
		Short: "Output shell completions for fish",
		RunE: func(cmd *cobra.Command, args []string) error {
			return cmd.Root().GenFishCompletion(os.Stdout, true)
		},
	}
	return cmd
}
