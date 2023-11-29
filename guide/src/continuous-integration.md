# Running `mdbook` in Continuous Integration

There are a variety of services such as [GitHub Actions] or [GitLab CI/CD] which can be used to test and deploy your book automatically.

The following provides some general guidelines on how to configure your service to run mdBook.
Specific recipes can be found at the [Automated Deployment] wiki page.

[GitHub Actions]: https://docs.github.com/en/actions
[GitLab CI/CD]: https://docs.gitlab.com/ee/ci/
[Automated Deployment]: https://github.com/rust-lang/mdBook/wiki/Automated-Deployment

## Installing mdBook

There are several different strategies for installing mdBook.
The particular method depends on your needs and preferences.

### Pre-compiled binaries

Perhaps the easiest method is to use the pre-compiled binaries found on the [GitHub Releases page][releases].
A simple approach would be to use the popular `curl` CLI tool to download the executable:

```sh
mkdir bin
curl -sSL https://github.com/rust-lang/mdBook/releases/download/v0.4.36/mdbook-v0.4.36-x86_64-unknown-linux-gnu.tar.gz | tar -xz --directory=bin
bin/mdbook build
```

Some considerations for this approach:

* This is relatively fast, and does not necessarily require dealing with caching.
* This does not require installing Rust.
* Specifying a specific URL means you have to manually update your script to get a new version.
  This may be a benefit if you want to lock to a specific version.
  However, some users prefer to automatically get a newer version when they are published.
* You are reliant on the GitHub CDN being available.

[releases]: https://github.com/rust-lang/mdBook/releases

### Building from source

Building from source will require having Rust installed.
Some services have Rust pre-installed, but if your service does not, you will need to add a step to install it.

After Rust is installed, `cargo install` can be used to build and install mdBook.
We recommend using a SemVer version specifier so that you get the latest **non-breaking** version of mdBook.
For example:

```sh
cargo install mdbook --no-default-features --features search --vers "^0.4" --locked
```

This includes several recommended options:

* `--no-default-features` — Disables features like the HTTP server used by `mdbook serve` that is likely not needed on CI.
  This will speed up the build time significantly.
* `--features search` — Disabling default features means you should then manually enable features that you want, such as the built-in [search] capability.
* `--vers "^0.4"` — This will install the most recent version of the `0.4` series.
  However, versions after like `0.5.0` won't be installed, as they may break your build.
  Cargo will automatically upgrade mdBook if you have an older version already installed.
* `--locked` — This will use the dependencies that were used when mdBook was released.
  Without `--locked`, it will use the latest version of all dependencies, which may include some fixes since the last release, but may also (rarely) cause build problems.

You will likely want to investigate caching options, as building mdBook can be somewhat slow.

[search]: guide/reading.md#search

## Running tests

You may want to run tests using [`mdbook test`] every time you push a change or create a pull request.
This can be used to validate Rust code examples in the book.

This will require having Rust installed.
Some services have Rust pre-installed, but if your service does not, you will need to add a step to install it.

Other than making sure the appropriate version of Rust is installed, there's not much more than just running `mdbook test` from the book directory.

You may also want to consider running other kinds of tests, like [mdbook-linkcheck] which will check for broken links.
Or if you have your own style checks, spell checker, or any other tests it might be good to run them in CI.

[`mdbook test`]: cli/test.md
[mdbook-linkcheck]: https://github.com/Michael-F-Bryan/mdbook-linkcheck#continuous-integration

## Deploying

You may want to automatically deploy your book.
Some may want to do this every time a change is pushed, and others may want to only deploy when a specific release is tagged.

You'll also need to understand the specifics on how to push a change to your web service.
For example, [GitHub Pages] just requires committing the output onto a specific git branch.
Other services may require using something like SSH to connect to a remote server.

The basic outline is that you need to run `mdbook build` to generate the output, and then transfer the files (which are in the `book` directory) to the correct location.

You may then want to consider if you need to invalidate any caches on your web service.

See the [Automated Deployment] wiki page for examples of various different services.

[GitHub Pages]: https://docs.github.com/en/pages

### 404 handling

mdBook automatically generates a 404 page to be used for broken links.
The default output is a file named `404.html` at the root of the book.
Some services like [GitHub Pages] will automatically use this page for broken links.
For other services, you may want to consider configuring the web server to use this page as it will provide the reader navigation to get back to the book.

If your book is not deployed at the root of the domain, then you should set the [`output.html.site-url`] setting so that the 404 page works correctly.
It needs to know where the book is deployed in order to load the static files (like CSS) correctly.
For example, this guide is deployed at <https://rust-lang.github.io/mdBook/>, and the `site-url` setting is configured like this:

```toml
# book.toml
[output.html]
site-url = "/mdBook/"
```

You can customize the look of the 404 page by creating a file named `src/404.md` in your book.
If you want to use a different filename, you can set [`output.html.input-404`] to a different filename.

[`output.html.site-url`]: format/configuration/renderers.md#html-renderer-options
[`output.html.input-404`]: format/configuration/renderers.md#html-renderer-options
