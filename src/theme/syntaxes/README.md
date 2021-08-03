# Note

This folder is not copied over to the book directory when using `mdbook init`, nor is it indexed at runtime. To add new syntaxes, first add a `.sublime-syntax` file in this directory, then run (from inside this directory):
```shell
$ ./gen-syntaxcache.sh
```
Make sure you have [`bat`](https://github.com/sharkdp/bat) installed, otherwise this won't work.

Don't worry if bat says:
```
No themes were found in './themes', using the default set
```

Now, you may rebuild the `mdBook` binary and the new syntaxes should be included.

This only needs to be run when the syntaxes in this folder are updated.