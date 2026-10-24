# The completions command

The completions command is used to generate auto-completions for some common shells.
This means when you type `mdbook` in your shell, you can then press your shell's auto-complete key (usually the Tab key) and it may display what the valid options are, or finish partial input.

The completions first need to be installed for your shell:

```bash
# bash
mdbook completions bash > ~/.local/share/bash-completion/completions/mdbook
# oh-my-zsh
mdbook completions zsh > ~/.oh-my-zsh/completions/_mdbook
autoload -U compinit && compinit
```

The command prints a completion script for the given shell.
Run `mdbook completions --help` for a list of supported shells.

Where to place the completions depend on which shell you are using and your operating system.
Consult your shell's documentation for more information one where to place the script.
