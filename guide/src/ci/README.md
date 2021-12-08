# Running `mdbook` in Continuous Integration

While the following examples use Travis CI and GitHub Actions, their principles should
straightforwardly transfer to other continuous integration providers as well.

## Ensuring Your Book Builds and Tests Pass

### Using Travis CI

Here is a sample Travis CI `.travis.yml` configuration that ensures `mdbook
build` and `mdbook test` run successfully. The key to fast CI turnaround times
is caching `mdbook` installs, so that you aren't compiling `mdbook` on every CI
run.

```yaml
{{#include travis.yml::16}}
```

### Using GitHub Actions

Next is a sample for GitHub Actions `.github/workflows/main.yml` that ensures `mdbook build` and `mdbook test` run successfully.

```yaml
{{#include github.yml::19}}
```

## Deploying Your Book to GitHub Pages

Following these instructions will result in your book being published to GitHub
pages after a successful CI run on your repository's `main` branch.

### Using Travis CI

First, create a new GitHub "Personal Access Token" with the "public_repo"
permissions (or "repo" for private repositories). Go to your repository's Travis
CI settings page and add an environment variable named `GITHUB_TOKEN` that is
marked secure and *not* shown in the logs.

Whilst still in your repository's settings page, navigate to Options and change the
Source on GitHub pages to `gh-pages`.

Then, append this snippet to your `.travis.yml` and update the path to the
`book` directory:

```yaml
{{#include travis.yml:18:}}
```

That's it!

Note: Travis has a new [dplv2](https://blog.travis-ci.com/2019-08-27-deployment-tooling-dpl-v2-preview-release) configuration that is currently in beta. To use this new format, update your `.travis.yml` file to:

```yaml
{{#include travis-xenial.yml}}
```

### Using GitHub Actions

Extend `.github/workflows/main.yml` to the following in the `jobs` section:

```yaml
{{#include github.yml:20:}}
```

### Deploying to GitHub Pages manually

If your CI doesn't support GitHub pages, or you're deploying somewhere else
with integrations such as Github Pages:

*note: you may want to use different tmp dirs*:

```console
$> git worktree add /tmp/book gh-pages
$> mdbook build
$> rm -rf /tmp/book/* # this won't delete the .git directory
$> cp -rp book/* /tmp/book/
$> cd /tmp/book
$> git add -A
$> git commit 'new book message'
$> git push origin gh-pages
$> cd -
```

Or put this into a Makefile rule:

```makefile
.PHONY: deploy
deploy: book
	@echo "====> deploying to github"
	git worktree add /tmp/book gh-pages
	rm -rf /tmp/book/*
	cp -rp book/* /tmp/book/
	cd /tmp/book && \
		git add -A && \
		git commit -m "deployed on $(shell date) by ${USER}" && \
		git push origin gh-pages
```

## Deploying Your Book to GitLab Pages
Inside your repository's project root, create a file named `.gitlab-ci.yml` with the following contents:
```yml
stages:
    - deploy

pages:
  stage: deploy
  image: rust
  variables:
    CARGO_HOME: $CI_PROJECT_DIR/cargo
  before_script:
    - export PATH="$PATH:$CARGO_HOME/bin"
    - mdbook --version || cargo install mdbook
  script:
    - mdbook build -d public
  rules:
    - if: '$CI_COMMIT_REF_NAME == "master"'
  artifacts:
    paths:
      - public
  cache:
    paths:
      - $CARGO_HOME/bin
```

After you commit and push this new file, GitLab CI will run and your book will be available!
