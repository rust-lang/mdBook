use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Result, Error, ErrorKind};
use book::bookitem::BookItem;

/*
pub enum LineType {
    Blank,
    Header,
    Link(String, PathBuf), // Name, Path
    ListItem(String, PathBuf, i32), // Name, Path, Level
    Other,
}
*/

pub fn construct_bookitems(path: &PathBuf) -> Result<Vec<BookItem>> {
    let mut summary = String::new();
    try!(try!(File::open(path)).read_to_string(&mut summary));

    let top_items = try!(parse_level(&mut summary.split('\n').collect(), 0));



    Ok(top_items)
}

fn parse_level(summary: &mut Vec<&str>, current_level: i32) -> Result<Vec<BookItem>> {

    let mut items: Vec<BookItem> = vec![];

    loop {
        if summary.len() <= 0 { break }

        let level = try!(level(summary[0], 4));

        if current_level > level { break }
        else if current_level < level {
            items.last_mut().unwrap().sub_items = try!(parse_level(summary, level))
        }
        else {
            // Do the thing
            if let Some(item) = parse_line(summary[0].clone()) {
                items.push(item);
            }
            summary.remove(0);
        }
    }

    Ok(items)
}

fn level(line: &str, spaces_in_tab: i32) -> Result<i32> {
    let mut spaces = 0;
    let mut level = 0;

    for ch in line.chars() {
        match ch {
            ' ' => spaces += 1,
            '\t' => level += 1,
            _ => break,
        }
        if spaces >= spaces_in_tab {
            level += 1;
            spaces = 0;
        }
    }

    // If there are spaces left, there is an indentation error
    if spaces > 0 {
        return Err(Error::new(
            ErrorKind::Other,
            format!("There is an indentation error on line:\n\n{}", line)
            )
        )
    }

    Ok(level)
}


fn parse_line(l: &str) -> Option<BookItem> {
    let mut name;
    let mut path;
    // Remove leading and trailing spaces or tabs
    let line = l.trim_matches(|c: char| { c == ' ' || c == '\t' });

    if let Some(c) = line.chars().nth(0) {
        match c {
            // List item
            '-' | '*' => {
                let mut start_delimitor;
                let mut end_delimitor;

                // In the future, support for list item that is not a link
                // Not sure if I should error on line I can't parse or just ignore them...
                if let Some(i) = line.find('[') { start_delimitor = i; }
                else { return None }

                if let Some(i) = line[start_delimitor..].find("](") {
                    end_delimitor = start_delimitor +i;
                }
                else { return None }

                name = line[start_delimitor + 1 .. end_delimitor].to_string();

                start_delimitor = end_delimitor + 1;
                if let Some(i) = line[start_delimitor..].find(')') {
                    end_delimitor = start_delimitor + i;
                }
                else { return None }

                path = PathBuf::from(line[start_delimitor + 1 .. end_delimitor].to_string());

                return Some(BookItem::new(name, path))
            }
            _ => {}
        }
    }

    None
}
