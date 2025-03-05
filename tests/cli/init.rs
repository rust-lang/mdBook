use crate::cli::cmd::mdbook_cmd;
use crate::dummy_book::DummyBook;

use mdbook::config::Config;

/// Run `mdbook init` with `--force` to skip the confirmation prompts
#[test]
fn base_mdbook_init_can_skip_confirmation_prompts() {
    let temp = DummyBook::new().build().unwrap();

    // doesn't exist before
    assert!(!temp.path().join("book").exists());

    let mut cmd = mdbook_cmd();
    cmd.args(["init", "--force"]).current_dir(temp.path());
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\nAll done, no errors...\n"));

    let config = Config::from_disk(temp.path().join("book.toml")).unwrap();
    assert_eq!(config.book.title, None);

    assert!(!temp.path().join(".gitignore").exists());
}

/// Run `mdbook init` with `--title` without git config.
///
/// Regression test for https://github.com/rust-lang/mdBook/issues/2485
#[test]
fn no_git_config_with_title() {
    let temp = DummyBook::new().build().unwrap();

    // doesn't exist before
    assert!(!temp.path().join("book").exists());

    let mut cmd = mdbook_cmd();
    cmd.args(["init", "--title", "Example title"])
        .current_dir(temp.path())
        .env("GIT_CONFIG_GLOBAL", "")
        .env("GIT_CONFIG_NOSYSTEM", "1");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("\nAll done, no errors...\n"));

    let config = Config::from_disk(temp.path().join("book.toml")).unwrap();
    assert_eq!(config.book.title.as_deref(), Some("Example title"));
}
