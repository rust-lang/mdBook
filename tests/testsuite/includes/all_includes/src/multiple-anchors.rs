// This tests multiple anchor pairs that are meant to be included
fn main() {
    // ANCHOR: include-these
    println!("yes");
    // ANCHOR: not-these
    println!("yes");
    // ANCHOR_END: include-these
    println!("no");
    // ANCHOR: include-these
    println!("yes");
    // ANCHOR_END: include-these
    println!("no");
    // ANCHOR_END: not-these
    println!("no");
}
