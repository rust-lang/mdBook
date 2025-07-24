use super::*;

#[test]
fn section_number_has_correct_dotted_representation() {
    let inputs = vec![
        (vec![0], "0."),
        (vec![1, 3], "1.3."),
        (vec![1, 2, 3], "1.2.3."),
    ];

    for (input, should_be) in inputs {
        let section_number = SectionNumber(input).to_string();
        assert_eq!(section_number, should_be);
    }
}

    #[test]
    fn book_iter_iterates_over_sequential_items() {
        let sections = vec![
                BookItem::Chapter(Chapter {
                    name: String::from("Chapter 1"),
                    content: String::from("# Chapter 1"),
                    ..Default::default()
                }),
                BookItem::Separator,
            ];
        let book = Book::new_with_sections(sections);

        let should_be: Vec<_> = book.sections.iter().collect();

        let got: Vec<_> = book.iter().collect();

        assert_eq!(got, should_be);
    }

    #[test]
    fn for_each_mut_visits_all_items() {
        let sections = vec![
                BookItem::Chapter(Chapter {
                    name: String::from("Chapter 1"),
                    content: String::from("# Chapter 1"),
                    number: None,
                    path: Some(PathBuf::from("Chapter_1/index.md")),
                    source_path: Some(PathBuf::from("Chapter_1/index.md")),
                    parent_names: Vec::new(),
                    sub_items: vec![
                        BookItem::Chapter(Chapter::new(
                            "Hello World",
                            String::new(),
                            "Chapter_1/hello.md",
                            Vec::new(),
                        )),
                        BookItem::Separator,
                        BookItem::Chapter(Chapter::new(
                            "Goodbye World",
                            String::new(),
                            "Chapter_1/goodbye.md",
                            Vec::new(),
                        )),
                    ],
                }),
                BookItem::Separator,
            ];
        let mut book = Book::new_with_sections(sections);

        let num_items = book.iter().count();
        let mut visited = 0;

        book.for_each_mut(|_| visited += 1);

        assert_eq!(visited, num_items);
    }

    #[test]
    fn iterate_over_nested_book_items() {
        let sections = vec![
                BookItem::Chapter(Chapter {
                    name: String::from("Chapter 1"),
                    content: String::from("# Chapter 1"),
                    number: None,
                    path: Some(PathBuf::from("Chapter_1/index.md")),
                    source_path: Some(PathBuf::from("Chapter_1/index.md")),
                    parent_names: Vec::new(),
                    sub_items: vec![
                        BookItem::Chapter(Chapter::new(
                            "Hello World",
                            String::new(),
                            "Chapter_1/hello.md",
                            Vec::new(),
                        )),
                        BookItem::Separator,
                        BookItem::Chapter(Chapter::new(
                            "Goodbye World",
                            String::new(),
                            "Chapter_1/goodbye.md",
                            Vec::new(),
                        )),
                    ],
                }),
                BookItem::Separator,
            ];
        let book = Book::new_with_sections(sections);

        let got: Vec<_> = book.iter().collect();

        assert_eq!(got.len(), 5);

        // checking the chapter names are in the order should be sufficient here...
        let chapter_names: Vec<String> = got
            .into_iter()
            .filter_map(|i| match *i {
                BookItem::Chapter(ref ch) => Some(ch.name.clone()),
                _ => None,
            })
            .collect();
        let should_be: Vec<_> = vec![
            String::from("Chapter 1"),
            String::from("Hello World"),
            String::from("Goodbye World"),
        ];

        assert_eq!(chapter_names, should_be);
    }

