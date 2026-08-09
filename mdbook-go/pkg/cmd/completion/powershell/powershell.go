// Package powershell implements the `doclens completion powershell`
// subcommand: generate the PowerShell completion script.
package powershell

import (
	"os"

	"github.com/spf13/cobra"
)

func NewCommand() *cobra.Command {
	cmd := &cobra.Command{
		Args:  cobra.NoArgs,
		Use:   "powershell",
		Short: "Output shell completions for powershell",
		RunE: func(cmd *cobra.Command, args []string) error {
			return cmd.Root().GenPowerShellCompletion(os.Stdout)
		},
	}
	return cmd
}
