# The serve command

The `serve` command is useful when you want to preview your book. It also does hot reloading of the webpage whenever a file changes.
It achieves this by serving the books content over `localhost:3000` (unless otherwise configured, see below) and runs a websocket server on `localhost:3001` which triggers the reloads.
This preferred by many for writing books with mdbook because it allows for you to see the result of your work instantly after every file change.

#### Specify a directory

Like `watch`, `serve` can take a directory as an argument to use instead of
the current working directory.

```bash
mdbook serve path/to/book
```


#### Server options

`serve` has four options: the http port, the websocket port, the interface to serve on, and the public address of the server so that the browser may reach the websocket server.

For example: suppose you had an nginx server for SSL termination which has a public address of 192.168.1.100 on port 80 and proxied that to 127.0.0.1 on port 8000. To run use the nginx proxy do:

```bash
mdbook serve path/to/book -p 8000 -i 127.0.0.1 -a 192.168.1.100
```

If you were to want live reloading for this you would need to proxy the websocket calls through nginx as well from `192.168.1.100:<WS_PORT>` to `127.0.0.1:<WS_PORT>`. The `-w` flag allows for the websocket port to be configured.

#### --open

When you use the `--open` (`-o`) option, mdbook will open the book in your
your default web browser after starting the server.

#### --dest-dir

The `--dest-dir` (`-d`) option allows you to change the output directory for your book.

-----

***note:*** *the `serve` command has not gotten a lot of testing yet, there could be some rough edges. If you discover a problem, please report it [on Github](https://github.com/rust-lang-nursery/mdBook/issues)*
