#[cfg(test)]

use book::chapter::Chapter;
use book::toc::TocContent;

#[test]
fn it_should_produce_the_section_as_string() {
    let mut c = TocContent::default();
    c.section = Some(vec![1, 9, 4]);
    let result = c.section_as_string();
    let expected = "1.9.4.".to_string();
    assert_eq!(result, expected);
}
