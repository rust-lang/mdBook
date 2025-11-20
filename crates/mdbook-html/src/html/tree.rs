//! Tree data structure for representing a markdown document.
//!
//! [`MarkdownTreeBuilder::build`] is the primary entry point of this module.
//! It takes events from [`pulldown_cmark`], and generates a [`Tree`]
//! structure of [`Node`] elements. It also handles all the various
//! transformations that mdbook performs, such as creating header links.

use super::tokenizer::parse_html;
use super::{HtmlRenderOptions, hide_lines, wrap_rust_main};
use crate::utils::{id_from_content, unique_id};
use ego_tree::{NodeId, NodeRef, Tree};
use html5ever::tendril::StrTendril;
use html5ever::tokenizer::{TagKind, Token};
use html5ever::{LocalName, QualName};
use indexmap::IndexMap;
use mdbook_core::config::RustEdition;
use mdbook_core::static_regex;
use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event, LinkType, Tag, TagEnd};
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use tracing::{trace, warn};

/// Helper to create a [`QualName`].
macro_rules! attr_qual_name {
    ($name:expr) => {
        QualName::new(None, html5ever::ns!(), LocalName::from($name))
    };
}

/// A node in the [`Tree`].
#[derive(Debug)]
pub(crate) enum Node {
    /// An HTML [`Element`].
    Element(Element),
    /// Plain text.
    ///
    /// This will be escaped when serialized.
    Text(StrTendril),
    /// An HTML comment.
    Comment(StrTendril),
    /// Root node of a tree fragment.
    ///
    /// This is a general purpose node whenever it is convenient to have a
    /// container of other nodes.
    Fragment,
    /// Raw data that should be copied into the output as-is without escaping.
    RawData(StrTendril),
}

impl Node {
    /// If this is an [`Element`], return it.
    pub(crate) fn as_element(&self) -> Option<&Element> {
        if let Node::Element(el) = self {
            Some(el)
        } else {
            None
        }
    }

    /// If this is an [`Element`], return it (mutable).
    fn as_element_mut(&mut self) -> Option<&mut Element> {
        if let Node::Element(el) = self {
            Some(el)
        } else {
            None
        }
    }
}

/// An HTML element.
#[derive(Debug)]
pub(crate) struct Element {
    /// The tag name.
    pub(crate) name: QualName,
    /// Element attributes.
    pub(crate) attrs: Attributes,
    /// True if this tag ends with `/>`.
    pub(crate) self_closing: bool,
    /// True if this was raw HTML written in the markdown.
    pub(crate) was_raw: bool,
}

impl Element {
    /// Creates a new HTML element.
    pub(crate) fn new(tag_name: &str) -> Element {
        let name = QualName::new(None, html5ever::ns!(html), LocalName::from(tag_name));
        Element {
            name,
            attrs: Attributes::new(),
            self_closing: false,
            was_raw: false,
        }
    }

    /// The name of this element.
    pub(crate) fn name(&self) -> &str {
        self.name.local.deref()
    }

    /// If this is a heading element, returns the level of the heading.
    #[allow(dead_code, reason = "currently only used in search")]
    pub(crate) fn heading_level(&self) -> Option<u8> {
        let name = self.name();
        if matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
            Some(name.as_bytes()[1] - b'0')
        } else {
            None
        }
    }

    /// Returns the value of an attribute.
    pub(crate) fn attr(&self, name: &str) -> Option<&str> {
        let qname = attr_qual_name!(name);
        self.attrs.get(&qname).map(Deref::deref)
    }

    /// Inserts an attribute.
    pub(crate) fn insert_attr(&mut self, name: &str, value: StrTendril) {
        let name = attr_qual_name!(name);
        self.attrs.insert(name, value);
    }
}

/// A map of attributes on an [`Element`].
type Attributes = IndexMap<QualName, StrTendril>;

/// Helper to convert [`CowStr`] to a [`StrTendril`].
trait ToTendril {
    /// Converts self to a [`StrTendril`].
    fn into_tendril(self) -> StrTendril;
}

impl ToTendril for CowStr<'_> {
    fn into_tendril(self) -> StrTendril {
        match self {
            CowStr::Boxed(s) => {
                let s: String = s.into();
                StrTendril::from(s)
            }
            CowStr::Borrowed(s) => StrTendril::from(s),
            CowStr::Inlined(s) => StrTendril::from(s.as_ref()),
        }
    }
}

/// Tracks the current state of parsing a table.
///
/// This is used to determine if it should generate `<th>` or `<td>` tags.
enum TableState {
    /// Currently in the table head.
    Head,
    /// Currently in the table body.
    Body,
}

/// A builder used to create a [`Tree`] of [`Node`] elements.
///
/// Parts of this are based on pulldown-cmark's serializer (like table handling).
pub(crate) struct MarkdownTreeBuilder<'opts, 'event, EventIter> {
    /// [`pulldown_cmark`] iterator of [`pulldown_cmark::Event`] elements.
    events: EventIter,
    /// Options for how to generate the HTML.
    options: &'opts HtmlRenderOptions<'opts>,
    /// The tree that is being built.
    tree: Tree<Node>,
    /// The ID of the current [`Node`].
    current_node: NodeId,
    /// The tag stack.
    ///
    /// This is used to set the `current_node` as the parser enters and leaves
    /// a tag.
    tag_stack: Vec<NodeId>,
    /// When parsing a table, whether or not we are currently in the head or
    /// the body.
    table_state: TableState,
    /// When parsing a table, the alignments of the columns.
    ///
    /// The count should match the number of columns.
    table_alignments: Vec<Alignment>,
    /// What parsing a table, the index of the current column.
    table_cell_index: usize,
    /// Mapping of footnote numbers.
    ///
    /// This is used for generating linkbacks in the definitions.
    ///
    /// This is a map of `name -> (number, count)`.
    ///
    /// - `name` is the name of the footnote.
    /// - `number` is the footnote number displayed in the output.
    /// - `count` is the number of references to this footnote (used for multiple
    ///   linkbacks, and checking for unused footnotes).
    footnote_numbers: HashMap<CowStr<'event>, (usize, u32)>,
    /// Footnote definitions.
    ///
    /// This is a map of `name -> NodeId` of each footnote definition. When
    /// parsing footnotes, they are initially left in the position where they
    /// were defined as an `<li>` tag. The [`NodeId`] here is the id of that
    /// tag. After the document has been parsed, all the definitions are moved
    /// to the end of the document.
    footnote_defs: HashMap<CowStr<'event>, NodeId>,
}

impl<'opts, 'event, EventIter> MarkdownTreeBuilder<'opts, 'event, EventIter>
where
    EventIter: Iterator<Item = Event<'event>>,
{
    /// Processes a [`pulldown_cmark`] iterator of [`pulldown_cmark::Event`]
    /// values, and generates a tree of [`Node`] values.
    pub(crate) fn build(options: &'opts HtmlRenderOptions<'opts>, events: EventIter) -> Tree<Node> {
        let tree = Tree::new(Node::Fragment);
        let root = tree.root().id();

        let mut builder = Self {
            events,
            options,
            tree,
            current_node: root,
            tag_stack: vec![root],
            table_state: TableState::Head,
            table_alignments: Vec::new(),
            table_cell_index: 0,
            footnote_numbers: HashMap::new(),
            footnote_defs: HashMap::new(),
        };
        builder.process_events();
        builder.add_header_links();
        builder.update_code_blocks();
        builder.convert_fontawesome();
        builder.tree
    }

    /// Append a new child to the current node.
    ///
    /// Returns the [`NodeId`] of the new node.
    fn append(&mut self, node: Node) -> NodeId {
        self.tree
            .get_mut(self.current_node)
            .unwrap()
            .append(node)
            .id()
    }

    /// Appends text to the current node.
    ///
    /// If the previous sibling is a text node, then it merges with that node.
    /// This makes some processing more convenient.
    fn append_text(&mut self, text: StrTendril) {
        let mut current = self.tree.get_mut(self.current_node).unwrap();
        if let Some(mut prev) = current.last_child()
            && let Node::Text(prev_text) = prev.value()
        {
            prev_text.push_slice(&text);
        } else {
            self.append(Node::Text(text));
        }
    }

    /// Append a new child to the current node, and make the new node the current node.
    ///
    /// This should only be used if you expect `pop` to be called.
    fn push(&mut self, node: Node) {
        let new_node = self.append(node);
        self.tag_stack.push(new_node);
        self.current_node = new_node;
    }

    /// Append a new child to the current node, and make the new node the current node.
    ///
    /// As compared to `push`, it is *not* expected that there will be a `pop` called
    /// for this node. The next call to `pop` will unwind the stack past this node.
    fn push_no_stack(&mut self, node: Node) {
        let new_node = self.append(node);
        self.current_node = new_node;
    }

    /// Switch the current node to the current node's parent.
    fn pop(&mut self) {
        self.tag_stack.pop();
        if let Some(&parent) = self.tag_stack.last() {
            self.current_node = parent;
        } else {
            panic!("pop too far processing `{}`", self.options.path.display());
        }
    }

    /// Returns all of the [`NodeId`]s, filtering out just the [`Element`]
    /// nodes where the given callback returns `true` based on the element
    /// name.
    fn node_ids_for_tag(&self, filter: &dyn Fn(&str) -> bool) -> Vec<NodeId> {
        self.tree
            .nodes()
            .filter(|node| {
                let Node::Element(el) = node.value() else {
                    return false;
                };
                filter(el.name())
            })
            .map(|node| node.id())
            .collect()
    }

    /// The main processing loop. Processes all events until the end.
    fn process_events(&mut self) {
        while let Some(event) = self.events.next() {
            trace!("event={event:?}");
            match event {
                Event::Start(tag) => self.start_tag(tag),
                Event::End(tag) => self.end_tag(tag),
                Event::Text(text) => {
                    self.append_text(text.into_tendril());
                }
                Event::Code(code) => {
                    self.push(Node::Element(Element::new("code")));
                    self.append(Node::Text(code.into_tendril()));
                    self.pop();
                }
                Event::InlineMath(text) => {
                    let mut span = Element::new("span");
                    span.insert_attr("class", "math math-inline".into());
                    self.push(Node::Element(span));
                    self.append(Node::Text(text.into_tendril()));
                    self.pop();
                }
                Event::DisplayMath(text) => {
                    let mut span = Element::new("span");
                    span.insert_attr("class", "math math-display".into());
                    self.push(Node::Element(span));
                    self.append(Node::Text(text.into_tendril()));
                    self.pop();
                }
                Event::Html(html) => {
                    // The loop in Tag::HtmlBlock should have consumed all
                    // Html events.
                    panic!(
                        "`{}` unexpected Html event: {html}",
                        self.options.path.display()
                    );
                }
                Event::InlineHtml(html) => self.append_html(&html),
                Event::FootnoteReference(name) => self.footnote_reference(name),
                Event::SoftBreak => {
                    self.append_text("\n".into());
                }
                Event::HardBreak => {
                    self.append(Node::Element(Element::new("br")));
                }
                Event::Rule => {
                    self.append(Node::Element(Element::new("hr")));
                }
                Event::TaskListMarker(checked) => {
                    let mut input = Element::new("input");
                    input.insert_attr("disabled", "".into());
                    input.insert_attr("type", "checkbox".into());
                    if checked {
                        input.insert_attr("checked", "".into());
                    }
                    self.push(Node::Element(input));
                    // Add some space before whatever follows.
                    self.append(Node::Text(" ".into()));
                    self.pop();
                }
            }
        }
        self.finish_stack();
        self.collect_footnote_defs();
    }

    fn start_tag(&mut self, tag: Tag<'event>) {
        let element = match tag {
            Tag::Paragraph => Element::new("p"),
            Tag::Heading {
                level,
                id,
                classes,
                attrs,
            } => {
                let mut el = Element::new(&level.to_string());
                for (name, value) in attrs {
                    let name =
                        QualName::new(None, html5ever::ns!(), LocalName::from(Cow::from(name)));
                    let value = value.unwrap_or_else(|| CowStr::from(""));
                    el.attrs.insert(name, value.into_tendril());
                }
                if let Some(id) = id {
                    el.insert_attr("id", id.into_tendril());
                }
                if !classes.is_empty() {
                    let classes = classes.join(" ");
                    el.insert_attr("class", classes.into());
                }
                el
            }
            Tag::BlockQuote(kind) => {
                let mut b = Element::new("blockquote");
                if let Some(kind) = kind {
                    let (class_kind, icon, text) = super::admonitions::select_tag(kind);
                    let class = format!("blockquote-tag blockquote-tag-{class_kind}");
                    b.insert_attr("class", class.into());
                    self.push(Node::Element(b));

                    let mut title = Element::new("p");
                    title.insert_attr("class", "blockquote-tag-title".into());
                    self.push(Node::Element(title));

                    let mut svg = Element::new("svg");
                    svg.insert_attr("viewbox", "0 0 16 16".into());
                    svg.insert_attr("width", "18".into());
                    svg.insert_attr("height", "18".into());
                    self.push(Node::Element(svg));
                    self.append_html(icon);
                    self.pop();

                    self.append(Node::Text(text.into()));

                    self.pop();
                    return;
                }
                b
            }
            Tag::CodeBlock(kind) => {
                let mut code = Element::new("code");
                match kind {
                    CodeBlockKind::Fenced(info) => {
                        let mut infos =
                            info.split([' ', '\t', ',']).filter(|info| !info.is_empty());
                        if let Some(lang) = infos.next() {
                            let mut classes = String::with_capacity(info.len() + 10);
                            // The first element in the infostring is treated as the language.
                            classes.push_str("language-");
                            classes.push_str(lang);
                            // The rest are just added as classes.
                            while let Some(info) = infos.next() {
                                classes.push(' ');
                                classes.push_str(info);
                            }
                            code.insert_attr("class", classes.into());
                        }
                    }
                    CodeBlockKind::Indented => {}
                }
                self.push_no_stack(Node::Element(Element::new("pre")));
                code
            }
            Tag::HtmlBlock => {
                // To process the HTML correctly, this needs to
                // collect it all into a single string.
                let mut html = String::new();
                while let Some(event) = self.events.next() {
                    match event {
                        Event::Html(text) | Event::Text(text) => html.push_str(&text),
                        Event::End(TagEnd::HtmlBlock) => break,
                        _ => panic!(
                            "`{}` unexpected event in html block {event:?}",
                            self.options.path.display()
                        ),
                    }
                }
                self.append_html(&html);
                // TagEnd::HtmlBlock must not pop.
                return;
            }
            Tag::List(Some(start)) => {
                let mut ol = Element::new("ol");
                if start != 1 {
                    ol.insert_attr("start", format!("{start}").into());
                }
                ol
            }
            Tag::List(None) => Element::new("ul"),
            Tag::Item => Element::new("li"),
            Tag::FootnoteDefinition(name) => {
                if self.footnote_defs.contains_key(&name) {
                    warn!(
                        "footnote `{name}` in {} defined multiple times - \
                                     not updating to new definition",
                        self.options.path.display()
                    );
                    self.eat_till_end();
                    return;
                } else {
                    let mut el = Element::new("li");
                    el.insert_attr("id", format!("footnote-{name}").into());
                    self.push(Node::Element(el));
                    self.footnote_defs.insert(name, self.current_node);
                    return;
                }
            }
            Tag::DefinitionList => Element::new("dl"),
            Tag::DefinitionListTitle => Element::new("dt"),
            Tag::DefinitionListDefinition => Element::new("dd"),
            Tag::Table(alignments) => {
                self.table_alignments = alignments.clone();
                // This div wrapper around the table is used to apply
                // `overflow-x: auto` so that wide tables can be scrolled
                // horizontally, rather than overflowing or scrolling the
                // entire page. See
                // https://github.com/rust-lang/mdBook/pull/1617
                let mut div = Element::new("div");
                div.insert_attr("class", "table-wrapper".into());
                self.push_no_stack(Node::Element(div));
                Element::new("table")
            }
            Tag::TableHead => {
                self.table_state = TableState::Head;
                self.table_cell_index = 0;
                let thead = Element::new("thead");
                self.push_no_stack(Node::Element(thead));
                Element::new("tr")
            }
            Tag::TableRow => {
                self.table_cell_index = 0;
                Element::new("tr")
            }
            Tag::TableCell => {
                let mut cell = match self.table_state {
                    TableState::Head => Element::new("th"),
                    TableState::Body => Element::new("td"),
                };
                let style = match self.table_alignments.get(self.table_cell_index) {
                    Some(&Alignment::Left) => "text-align: left",
                    Some(&Alignment::Center) => "text-align: center",
                    Some(&Alignment::Right) => "text-align: right",
                    Some(&Alignment::None) | None => "",
                };
                if !style.is_empty() {
                    cell.insert_attr("style", style.into());
                }
                cell
            }
            Tag::Emphasis => Element::new("em"),
            Tag::Strong => Element::new("strong"),
            Tag::Strikethrough => Element::new("del"),
            Tag::Superscript => Element::new("sup"),
            Tag::Subscript => Element::new("sub"),
            Tag::Link {
                link_type,
                dest_url,
                title,
                id: _,
            } => {
                let href: StrTendril = if matches!(link_type, LinkType::Email) {
                    format!("mailto:{dest_url}").into()
                } else {
                    fix_link(dest_url).into_tendril()
                };
                let mut a = Element::new("a");
                a.insert_attr("href", href);
                if !title.is_empty() {
                    a.insert_attr("title", title.into_tendril());
                }
                a
            }
            Tag::Image {
                link_type: _,
                dest_url,
                title,
                id: _,
            } => {
                let mut img = Element::new("img");
                let src = fix_link(dest_url).into_tendril();
                img.insert_attr("src", src);
                if !title.is_empty() {
                    img.insert_attr("title", title.into_tendril());
                }
                // This will eat TagEnd::Image
                let alt = self.text_for_img_alt();
                img.insert_attr("alt", alt.into());
                self.append(Node::Element(img));
                return;
            }
            Tag::MetadataBlock(_) => {
                // Eat all events till the end of MetadataBlock.
                while let Some(event) = self.events.next() {
                    if matches!(event, Event::End(TagEnd::MetadataBlock(_))) {
                        break;
                    }
                }
                return;
            }
        };
        self.push(Node::Element(element));
    }

    fn end_tag(&mut self, tag: TagEnd) {
        // TODO: This should validate that the event stack is properly
        // synchronized with the tag stack. That, would likely require keeping
        // a parallel "expected end tag" with the tag stack, since mapping a
        // pulldown-cmark event tag to an HTML tag isn't always clear.
        //
        // Check for unclosed HTML tags when exiting a markdown event.
        while let Some(node_id) = self.tag_stack.last() {
            let node = self.tree.get(*node_id).unwrap().value();
            let Node::Element(el) = node else {
                break;
            };
            if !el.was_raw {
                break;
            }
            warn!(
                "unclosed HTML tag `<{}>` found in `{}` while exiting {tag:?}\n\
                HTML tags must be closed before exiting a markdown element.",
                el.name.local,
                self.options.path.display(),
            );
            self.pop();
        }
        self.pop();
        match tag {
            TagEnd::TableHead => {
                self.table_state = TableState::Body;
                self.push(Node::Element(Element::new("tbody")));
            }
            TagEnd::TableCell => {
                self.table_cell_index += 1;
            }
            TagEnd::Table => {
                // Pop tbody or thead
                self.pop();
            }
            _ => {}
        }
    }

    /// Given some HTML, parse it into [`Node`] elements and append them to
    /// the current node.
    fn append_html(&mut self, html: &str) {
        let tokens = parse_html(&html);
        let mut is_raw = false;
        for token in tokens {
            trace!("html token={token:?}");
            match token {
                Token::DoctypeToken(_) => {}
                Token::TagToken(tag) => match tag.kind {
                    TagKind::StartTag => self.start_html_tag(tag, &mut is_raw),
                    TagKind::EndTag => self.end_html_tag(tag, &mut is_raw),
                },
                Token::CommentToken(comment) => {
                    self.append(Node::Comment(comment));
                }
                Token::CharacterTokens(chars) => {
                    if is_raw {
                        self.append(Node::RawData(chars));
                    } else {
                        self.append_text(chars);
                    }
                }
                Token::NullCharacterToken => {}
                Token::EOFToken => {}
                Token::ParseError(error) => {
                    warn!(
                        "html parse error in `{}`: {error}\n\
                         Html text was:\n\
                         {html}",
                        self.options.path.display()
                    );
                }
            }
        }
    }

    /// Adds an open HTML tag.
    fn start_html_tag(&mut self, tag: html5ever::tokenizer::Tag, is_raw: &mut bool) {
        let is_closed = is_void_element(&tag.name) || tag.self_closing;
        *is_raw = matches!(&*tag.name, "script" | "style");
        let name = QualName::new(None, html5ever::ns!(html), tag.name);
        let attrs = tag
            .attrs
            .into_iter()
            .map(|attr| (attr.name, attr.value))
            .collect();
        let mut el = Element {
            name,
            attrs,
            self_closing: tag.self_closing,
            was_raw: true,
        };
        fix_html_link(&mut el);
        self.push(Node::Element(el));
        if is_closed {
            // No end element.
            self.pop();
        }
    }

    /// Closes the given HTML tag.
    fn end_html_tag(&mut self, tag: html5ever::tokenizer::Tag, is_raw: &mut bool) {
        *is_raw = false;
        if self.is_html_tag_matching(&tag.name) {
            self.pop();
        } else {
            // The proper thing to do here is to recover. However, the HTML
            // parsing algorithm for that is quite complex. See
            // https://html.spec.whatwg.org/multipage/parsing.html#parsing-main-inbody
            // and the adoption agency algorithm.
            warn!(
                "unexpected HTML end tag `</{}>` found in `{}`\n\
                 Check that the HTML tags are properly balanced.",
                tag.name,
                self.options.path.display()
            );
        }
    }

    /// This is used to verify HTML parsing keeps the stack of tags in sync.
    fn is_html_tag_matching(&self, name: &str) -> bool {
        let current = self.tree.get(self.current_node).unwrap().value();
        if let Node::Element(el) = current
            && el.name() == name
        {
            true
        } else {
            false
        }
    }

    /// Eats all pulldown-cmark events until the next `End` matching the
    /// current nesting level.
    fn eat_till_end(&mut self) {
        let mut nest = 0;
        while let Some(event) = self.events.next() {
            match event {
                Event::Start(_) => nest += 1,
                Event::End(_) => {
                    if nest == 0 {
                        break;
                    }
                    nest -= 1;
                }
                _ => {}
            }
        }
    }

    /// Eats events generating a plain text string, stripping out any
    /// formatting elements.
    fn text_for_img_alt(&mut self) -> String {
        let mut nest = 0;
        let mut output = String::new();
        while let Some(event) = self.events.next() {
            match event {
                Event::Start(_) => nest += 1,
                Event::End(_) => {
                    if nest == 0 {
                        break;
                    }
                    nest -= 1;
                }
                Event::Html(_) => {}
                Event::InlineHtml(text) | Event::Code(text) | Event::Text(text) => {
                    output.push_str(&text);
                }
                Event::InlineMath(text) => {
                    output.push('$');
                    output.push_str(&text);
                    output.push('$');
                }
                Event::DisplayMath(text) => {
                    output.push_str("$$");
                    output.push_str(&text);
                    output.push_str("$$");
                }
                Event::SoftBreak | Event::HardBreak | Event::Rule => output.push(' '),
                Event::FootnoteReference(_) => {}
                Event::TaskListMarker(_) => {}
            }
        }
        output
    }

    /// Deals with any unclosed elements on the stack.
    fn finish_stack(&mut self) {
        while let Some(node_id) = self.tag_stack.pop() {
            let node = self.tree.get(node_id).unwrap().value();
            match node {
                Node::Fragment => {}
                Node::Element(el) => {
                    if el.was_raw {
                        warn!(
                            "unclosed HTML tag `<{}>` found in `{}`",
                            el.name.local,
                            self.options.path.display()
                        );
                    } else {
                        panic!(
                            "internal error: expected empty tag stack.\n
                             path: `{}`\n\
                             element={el:?}",
                            self.options.path.display()
                        );
                    }
                }
                node => {
                    panic!(
                        "internal error: expected empty tag stack.\n
                         path: `{}`\n\
                         node={node:?}",
                        self.options.path.display()
                    );
                }
            }
        }
    }

    /// Appends a new footnote reference.
    fn footnote_reference(&mut self, name: CowStr<'event>) {
        let len = self.footnote_numbers.len() + 1;
        let (n, count) = self
            .footnote_numbers
            .entry(name.clone())
            .or_insert((len, 0));
        *count += 1;
        let (n, count) = (*n, *count);

        let current = self.tree.get(self.current_node).unwrap();
        if let Some(last) = current.last_child()
            && let Node::Element(el) = last.value()
        {
            if el.attr("class") == Some("footnote-reference") {
                self.append(Node::Text(" ".into()));
            }
        }
        let mut sup = Element::new("sup");
        sup.insert_attr("class", "footnote-reference".into());
        let id = format!("fr-{name}-{count}");
        sup.insert_attr("id", id.into());
        self.push(Node::Element(sup));
        let mut a = Element::new("a");
        a.insert_attr("href", format!("#footnote-{name}").into());
        self.push(Node::Element(a));
        self.append(Node::Text(format!("{n}").into()));
        self.pop(); // a
        self.pop(); // sup
    }

    /// This is used after parsing is complete to move the footnote
    /// definitions to the end of the document.
    fn collect_footnote_defs(&mut self) {
        if self.footnote_defs.is_empty() {
            return;
        }
        let defs = std::mem::take(&mut self.footnote_defs);
        let mut defs: Vec<_> = defs.into_iter().collect();
        // Detach nodes and remove unused.
        defs.retain(|(name, def_id)| {
            let mut node = self.tree.get_mut(*def_id).unwrap();
            node.detach();

            if !self.footnote_numbers.contains_key(name) {
                warn!(
                    "footnote `{name}` in `{}` is defined but not referenced",
                    self.options.path.display()
                );
                false
            } else {
                true
            }
        });
        defs.sort_by_cached_key(|(name, _)| self.footnote_numbers[name].0);

        // Move defs to the end of the chapter.
        self.append(Node::Element(Element::new("hr")));
        let mut ol = Element::new("ol");
        ol.insert_attr("class", "footnote-definition".into());
        let ol_id = self.append(Node::Element(ol));
        for (name, def_id) in defs {
            // Generate the linkbacks.
            let count = self.footnote_numbers[&name].1;
            for usage in 1..=count {
                let nth = if usage == 1 {
                    String::new()
                } else {
                    usage.to_string()
                };
                let space = self.tree.orphan(Node::Text(" ".into())).id();
                let mut backlink = Element::new("a");
                backlink.insert_attr("href", format!("#fr-{name}-{usage}").into());
                let mut backlink = self.tree.orphan(Node::Element(backlink));
                backlink.append(Node::Text(format!("↩{nth}").into()));
                let backlink = backlink.id();
                let mut def = self.tree.get_mut(def_id).unwrap();
                if let Some(mut last_child) = def.last_child()
                    && let value = last_child.value()
                    && let Node::Element(last_el) = value
                    && last_el.name() == "p"
                {
                    // Put the linkback at the end of the last paragraph instead
                    // of on a line by itself.
                    last_child.append_id(space);
                    last_child.append_id(backlink);
                } else {
                    // Not a clear place to put it in this circumstance, so put it
                    // at the end.
                    def.append_id(space);
                    def.append_id(backlink);
                };
            }
            let mut ol = self.tree.get_mut(ol_id).unwrap();
            ol.append_id(def_id);
        }
    }

    /// This is used after parsing is complete to add a unique `id` attribute
    /// to all header and dt elements, and to also add an `<a>` tag so that
    /// clicking the element will set the current URL to that element's
    /// fragment.
    fn add_header_links(&mut self) {
        let mut id_counter = HashSet::new();
        let headings = self.node_ids_for_tag(&|name| {
            matches!(name, "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "dt")
        });
        for heading in headings {
            let node = self.tree.get(heading).unwrap();
            let el = node.value().as_element().unwrap();
            // Don't modify tags if they were manually written HTML. The
            // user probably had some intent, and we don't want to mess it up.
            if el.was_raw {
                continue;
            }
            let href = if let Some(id) = el.attr("id") {
                format!("#{id}")
            } else {
                let mut id = String::new();
                let node_id = node.id();
                let node_ref = self.tree.get(node_id).unwrap();
                text_in_node(node_ref, &mut id);
                let id = id_from_content(&id);
                let id = unique_id(&id, &mut id_counter);
                let mut node = self.tree.get_mut(heading).unwrap();
                let el = node.value().as_element_mut().unwrap();
                let href = format!("#{id}");
                el.insert_attr("id", id.into());
                href
            };
            // Insert an <a> element between the heading and its children.
            let mut a = Element::new("a");
            a.insert_attr("class", "header".into());
            a.insert_attr("href", href.into());
            let mut a = self.tree.orphan(Node::Element(a));
            a.reparent_from_id_append(heading);
            let a_id = a.id();
            let mut node = self.tree.get_mut(heading).unwrap();
            node.append_id(a_id);
        }
    }

    /// This is used after parsing is complete to set the appropriate classes
    /// on a code block, to wrap hidden lines in `<span>` tags, and to add an
    /// `fn main() {}` wrapper for Rust code blocks.
    fn update_code_blocks(&mut self) {
        let mut code_ids = self.node_ids_for_tag(&|name| name == "code");
        // The processing below assumes the code block is in a contiguous
        // chunk. The text nodes should have been merged during event
        // processing. I don't know exactly what this should do if it
        // encounters code blocks with non-text nodes.
        code_ids.retain(|id| {
            let code = self.tree.get(*id).unwrap();
            code.children().count() == 1
        });

        for code_id in code_ids.iter().copied() {
            let mut node = self.tree.get_mut(code_id).unwrap();
            let parent_id = node.parent().unwrap().id();
            let code_el = node.value().as_element_mut().unwrap();
            let class = code_el.attr("class").unwrap_or_default();
            let class_set: HashSet<_> = class.split(' ').collect();
            let is_editable = class_set.contains("editable");
            let is_playground = class_set.contains("language-rust")
                && ((!class_set.contains("ignore")
                    && !class_set.contains("noplayground")
                    && !class_set.contains("noplaypen")
                    && self.options.config.playground.runnable)
                    || class_set.contains("mdbook-runnable"));
            if !is_playground {
                continue;
            }
            let add_edition = if class_set.iter().any(|cls| cls.starts_with("edition")) {
                None
            } else {
                self.options.edition.map(|edition| match edition {
                    RustEdition::E2015 => "edition2015",
                    RustEdition::E2018 => "edition2018",
                    RustEdition::E2021 => "edition2021",
                    RustEdition::E2024 => "edition2024",
                    _ => panic!("edition {edition:?} not covered"),
                })
            };
            if let Some(edition) = add_edition {
                code_el.insert_attr("class", format!("{class} {edition}").into());
            }

            let mut node = self.tree.get_mut(code_id).unwrap();
            if !self.options.config.playground.editable || !is_editable {
                if let Some(mut child) = node.first_child()
                    && let Node::Text(text) = child.value()
                {
                    if let Some(new_text) = wrap_rust_main(text) {
                        *text = new_text.into();
                    }
                }
            }

            let mut pre = self.tree.get_mut(parent_id).unwrap();
            let pre = pre.value().as_element_mut().unwrap();
            assert_eq!(pre.name(), "pre");
            pre.insert_attr("class", "playground".into());
        }

        for code_id in code_ids {
            hide_lines(&mut self.tree, code_id, &self.options.config.code.hidelines);
        }
    }

    /// This is used after parsing is complete to replace `<i>` tags with a
    /// `<span>` that includes the corresponding SVG code.
    fn convert_fontawesome(&mut self) {
        use font_awesome_as_a_crate as fa;

        let is = self.node_ids_for_tag(&|name| name == "i");
        for i_id in is {
            let mut icon = String::new();
            let mut type_ = fa::Type::Regular;
            let mut new_classes = String::from("fa-svg");

            let mut node = self.tree.get_mut(i_id).unwrap();
            if node.first_child().is_some() {
                // Just to be safe, only translate <i></i>.
                continue;
            }
            let i_el = node.value().as_element().unwrap();
            let classes = i_el.attr("class").unwrap_or_default();
            for class in classes.split(" ") {
                if matches!(class, "fa" | "fa-regular") {
                    type_ = fa::Type::Regular;
                } else if matches!(class, "fas" | "fa-solid") {
                    type_ = fa::Type::Solid;
                } else if matches!(class, "fab" | "fa-brands") {
                    type_ = fa::Type::Brands;
                } else if let Some(class) = class.strip_prefix("fa-") {
                    icon = class.to_owned();
                } else {
                    new_classes += " ";
                    new_classes += class;
                }
            }
            if icon.is_empty() {
                continue;
            }

            match fa::svg(type_, &icon) {
                Ok(svg) => {
                    let mut span = Element::new("span");
                    span.insert_attr("class", new_classes.into());
                    for (name, value) in &i_el.attrs {
                        if *name != attr_qual_name!("class") {
                            span.attrs.insert(name.clone(), value.clone());
                        }
                    }
                    *node.value() = Node::Element(span);
                    node.append(Node::RawData(svg.into()));
                }
                Err(e) => {
                    warn!(
                        "failed to find Font Awesome icon for icon `{icon}` \
                         with type `{type_}` in `{}`: {e}",
                        self.options.path.display()
                    );
                }
            }
        }
    }
}

/// Traverse the given node, emitting any plain text into the output.
///
/// This is used to generate the `id` of a header.
fn text_in_node(node: NodeRef<'_, Node>, output: &mut String) {
    for child in node.children() {
        match child.value() {
            Node::Element(_) => {}
            Node::Text(text) => output.push_str(text),
            Node::Comment(_) => {}
            Node::Fragment => {}
            Node::RawData(_) => {}
        }
        text_in_node(child, output);
    }
}

/// Modifies links to work with HTML.
///
/// For local paths, this changes the `.md` extension to `.html`.
fn fix_link<'a>(link: CowStr<'a>) -> CowStr<'a> {
    static_regex!(SCHEME_LINK, r"^[a-z][a-z0-9+.-]*:");
    static_regex!(MD_LINK, r"(?P<link>.*)\.md(?P<anchor>#.*)?");

    if link.starts_with('#') {
        // Fragment-only link.
        return link;
    }
    // Don't modify links with schemes like `https`.
    if SCHEME_LINK.is_match(&link) {
        return link;
    }

    // This is a relative link, adjust it as necessary.
    if let Some(caps) = MD_LINK.captures(&link) {
        let mut fixed_link = String::from(&caps["link"]);
        fixed_link.push_str(".html");
        if let Some(anchor) = caps.name("anchor") {
            fixed_link.push_str(anchor.as_str());
        }
        CowStr::from(fixed_link)
    } else {
        link
    }
}

/// Calls [`fix_link`] for HTML elements.
fn fix_html_link(el: &mut Element) {
    if el.name() != "a" {
        return;
    }
    for attr in ["href", "xlink:href"] {
        if let Some(value) = el.attr(attr) {
            let fixed = fix_link(value.into());
            el.insert_attr(attr, fixed.into_tendril());
        }
    }
}

/// Whether or not this element name is a [void element].
///
/// This is used to know whether or not to expect a `</>` end tag.
///
/// [void element]: https://developer.mozilla.org/en-US/docs/Glossary/Void_element
pub(crate) fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}
