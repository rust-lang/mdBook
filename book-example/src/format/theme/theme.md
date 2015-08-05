# Theme

The default renderer uses a [handlebars](http://handlebarsjs.com/) template to render your markdown files in and comes with a default theme
included in the mdBook binary.

But the theme is totally customizable, you can replace every file from the theme by your own by adding a
`theme` directory in your source folder. Create a new file with the name of the file you want to overwrite
and now that file will be used instead of the default file.

Here are the files you can overwrite:

- ***index.hbs*** is the handlebars template.
- ***book.css*** is the style used in the output. If you want to change the design of your book, this is probably the file you want to modify. Sometimes in conjunction with `index.hbs` when you want to radically change the layout.
- ***book.js*** is mostly used to add client side functionality.

**Note:**

When you overwrite a file, it is possible that you break some functionality. Therefore I recommend to use the file from the default theme as template and only add / modify what you need. In the future you will be able to copy the default theme into your source directory automatically by using `mdbook init --theme`.
