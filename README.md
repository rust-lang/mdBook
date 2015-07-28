# mdBook

Personal implementation of Gitbook in Rust

**This is a work in progress, it's far from being usable at the moment...**

### Progress

- [x] `mdbook init` creates boilerplate directory structure and files to start with. <br><sup>Could be tweaked a little bit for improvements, but it works</sup>
- [x] Parses `SUMMARY.md` and constructs a book data structure.<br><sup>Supports nested levels, empty links. Does not support other lines than list elements, does not suppport plain text. (Does not support = ignore)</sup>
- [x] Create JSon data from book
- [x] render handlebars template to html
- [x] create one `html` file for every entry in `SUMMARY.md` that is not an empty link<br><sup>Respecting original directory structure</sup>
- [x] page layout
- [x] show content on page rendered from markdown
- [x] construct sidebar table of contents
- [ ] support config file

For more information about progress and what is still on my to-do list, check [this issue](../../issues/1)
