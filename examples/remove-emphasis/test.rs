#[test]
fn remove_emphasis_works() {
    // Tests that the remove-emphasis example works as expected.
    use mdbook::book::ActiveBackends;

    // Workaround for https://github.com/rust-lang/mdBook/issues/1424
    std::env::set_current_dir("examples/remove-emphasis").unwrap();
    let book = mdbook::MDBook::load(".").unwrap();
    book.render(&ActiveBackends::AllAvailable).unwrap();
    let ch1 = std::fs::read_to_string("book/chapter_1.html").unwrap();
    assert!(ch1.contains("This has light emphasis and bold emphasis."));
}
