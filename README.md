# mdBook

[![CI Status](https://github.com/rust-lang/mdBook/actions/workflows/main.yml/badge.svg)](https://github.com/rust-lang/mdBook/actions/workflows/main.yml)
[![crates.io](https://img.shields.io/crates/v/mdbook.svg)](https://crates.io/crates/mdbook)
[![LICENSE](https://img.shields.io/github/license/rust-lang/mdBook.svg)](LICENSE)

mdBook is a utility to create modern online books from Markdown files.

Check out the **[User Guide]** for a list of features and installation and usage information.
The User Guide also serves as a demonstration to showcase what a book looks like.

If you are interested in contributing to the development of mdBook, check out the [Contribution Guide].

## Container

> **NOTE**: You need to have docker installed
> https://docs.docker.com/engine/install/

1. Locate a local directory with md files
2. Quickly run a docker container with the current version:

```console
docker run -ti -v $(pwd):/mdbook -p 3000:3000 rust-lang/mdbook
```

* Using docker-compose.yaml, you can run to load the sources from the `test-book`.

```console
$ docker compose up --build
[+] Building 0.0s (0/0)                                                                                                                                              docker-container:mac-linux-builder
[+] Running 1/1
 ✔ Container rustlang-mdbook  Recreated                                                                                                                                                            0.1s 
Attaching to rustlang-mdbook
rustlang-mdbook  | 2023-11-23 16:52:59 [INFO] (mdbook::book): Book building has started
rustlang-mdbook  | 2023-11-23 16:52:59 [INFO] (mdbook::book): Running the html backend
rustlang-mdbook  | 2023-11-23 16:53:00 [INFO] (mdbook::cmd::serve): Serving on: http://0.0.0.0:3000
rustlang-mdbook  | 2023-11-23 16:53:00 [INFO] (warp::server): Server::run; addr=0.0.0.0:3000
rustlang-mdbook  | 2023-11-23 16:53:00 [INFO] (warp::server): listening on http://0.0.0.0:3000
rustlang-mdbook  | 2023-11-23 16:53:00 [INFO] (mdbook::cmd::watch): Listening for changes...
```

* Test it in a container as well

> **NOTE**: docker compose creates a default network with the name of the `dir_default` 

```console
docker run -ti --network mdbook_default alpine/lynx mdbook:3000
                                                                                                                                                                                  
Introduction (p1 of 3)
    1. Prefix Chapter
    2.
    3. 1. Introduction
    4. 2. Draft Chapter
    5.
    6. Actual Markdown Tag Examples
    7. 3. Markdown Individual tags
    8.
         1. 3.1. Heading
         2. 3.2. Paragraphs
         3. 3.3. Line Break
         4. 3.4. Emphasis
         5. 3.5. Blockquote
         6. 3.6. List
         7. 3.7. Code
         8. 3.8. Image
         9. 3.9. Links and Horizontal Rule
        10. 3.10. Tables
```

## License

All the code in this repository is released under the ***Mozilla Public License v2.0***, for more information take a look at the [LICENSE] file.

[User Guide]: https://rust-lang.github.io/mdBook/
[contribution guide]: https://github.com/rust-lang/mdBook/blob/master/CONTRIBUTING.md
[LICENSE]: https://github.com/rust-lang/mdBook/blob/master/LICENSE
