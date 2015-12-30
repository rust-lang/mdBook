use std::path::{Path, PathBuf};

pub fn render_playpen(s: &mut str) {

    for playpen in find_playpens(s) {
        println!("Playpen{{ {}, {}, {:?}, {} }}", playpen.start_index, playpen.end_index, playpen.rust_file, playpen.editable);
    }

}

struct Playpen{
    start_index: u32,
    end_index: u32,
    rust_file: PathBuf,
    editable: bool
}

fn find_playpens(s: &str) -> Vec<Playpen> {
    let mut playpens = vec![];
    for (i, _) in s.match_indices("{{#playpen") {
        println!("[*]: find_playpen");

        // DON'T forget the "+ i" else you have an index out of bounds error !!
        let end_i = if let Some(n) = s[i..].find("}}") { n } else { continue } + i + 2;

        println!("s[{}..{}] = {}", i, end_i, s[i..end_i].to_string());

        // If there is nothing between "{{#playpen" and "}}" skip
        if end_i-2 - (i+10) < 1 { continue }
        if s[i+10..end_i-2].trim().len() == 0 { continue }

        println!("{}", s[i+10..end_i-2].to_string());

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
                rust_file: PathBuf::from(params[0]),
                editable: editable,
            }
        )
    }

    playpens
}
