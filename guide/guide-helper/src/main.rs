//! Preprocessor for the mdBook guide.

fn main() {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("supports") => {
            // Supports all renderers.
            return;
        }
        Some(arg) => {
            eprintln!("unknown argument: {arg}");
            std::process::exit(1);
        }
        None => {}
    }

    if let Err(e) = guide_helper::handle_preprocessing() {
        eprintln!("{e:?}");
        std::process::exit(1);
    }
}
