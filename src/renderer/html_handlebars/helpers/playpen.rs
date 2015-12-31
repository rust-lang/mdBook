use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;

pub fn render_playpen(s: &mut str, path: &Path) {
    // When replacing one thing in a string by something with a different length, the indices
    // after that will not correspond, we therefore have to store the difference to correct this
    let difference_index = 0;

    for playpen in find_playpens(s, path) {

        // Check if the file exists
        if !playpen.rust_file.exists() || !playpen.rust_file.is_file() {
            output!("[-] No file exists for {{{{#playpen }}}}\n    {}", playpen.rust_file.to_str().unwrap());
            continue
        }

        // Open file & read file
        let mut file = if let Ok(f) = File::open(&playpen.rust_file) { f } else { continue };
        let mut file_content = String::new();
        if let Err(_) = file.read_to_string(&mut file_content) { continue };

        //println!("Playpen{{ {}, {}, {:?}, {} }}", playpen.start_index, playpen.end_index, playpen.rust_file, playpen.editable);
    }

}

#[derive(PartialOrd, PartialEq, Debug)]
struct Playpen{
    start_index: u32,
    end_index: u32,
    rust_file: PathBuf,
    editable: bool
}

fn find_playpens(s: &str, base_path: &Path) -> Vec<Playpen> {
    let mut playpens = vec![];
    for (i, _) in s.match_indices("{{#playpen") {
        debug!("[*]: find_playpen");

        // DON'T forget the "+ i" else you have an index out of bounds error !!
        let end_i = if let Some(n) = s[i..].find("}}") { n } else { continue } + i + 2;

        debug!("s[{}..{}] = {}", i, end_i, s[i..end_i].to_string());

        // If there is nothing between "{{#playpen" and "}}" skip
        if end_i-2 - (i+10) < 1 { continue }
        if s[i+10..end_i-2].trim().len() == 0 { continue }

        debug!("{}", s[i+10..end_i-2].to_string());

        // Split on whitespaces
        let params: Vec<&str> = s[i+10..end_i-2].split_whitespace().collect();
        let mut editable = false;

        if params.len() > 1 {
            editable = if let Some(_) = params[1].find("editable") {true} else {false};
        }

        playpens.push(
            Playpen{
                start_index: i as u32,
                end_index: end_i as u32,
                rust_file: base_path.join(PathBuf::from(params[0])),
                editable: editable,
            }
        )
    }

    playpens
}




//
//---------------------------------------------------------------------------------
//      Tests
//

#[test]
fn test_find_playpens_no_playpen() {
    let s = "Some random text without playpen...";
    assert!(find_playpens(s, Path::new("")) == vec![]);
}

#[test]
fn test_find_playpens_partial_playpen() {
    let s = "Some random text with {{#playpen...";
    assert!(find_playpens(s, Path::new("")) == vec![]);
}

#[test]
fn test_find_playpens_empty_playpen() {
    let s = "Some random text with {{#playpen}} and {{#playpen   }}...";
    assert!(find_playpens(s, Path::new("")) == vec![]);
}

#[test]
fn test_find_playpens_simple_playpen() {
    let s = "Some random text with {{#playpen file.rs}} and {{#playpen test.rs }}...";

    println!("\nOUTPUT: {:?}\n", find_playpens(s, Path::new("")));

    assert!(find_playpens(s, Path::new("")) == vec![
        Playpen{start_index: 22, end_index: 42, rust_file: PathBuf::from("file.rs"), editable: false},
        Playpen{start_index: 47, end_index: 68, rust_file: PathBuf::from("test.rs"), editable: false}
    ]);
}

#[test]
fn test_find_playpens_complex_playpen() {
    let s = "Some random text with {{#playpen file.rs editable}} and {{#playpen test.rs editable }}...";

    println!("\nOUTPUT: {:?}\n", find_playpens(s, Path::new("dir")));

    assert!(find_playpens(s, Path::new("dir")) == vec![
        Playpen{start_index: 22, end_index: 51, rust_file: PathBuf::from("dir/file.rs"), editable: true},
        Playpen{start_index: 56, end_index: 86, rust_file: PathBuf::from("dir/test.rs"), editable: true}
    ]);
}
