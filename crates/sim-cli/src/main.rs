//! `embed-emu` command-line entry point.

fn main() {
    println!("embed-emu {}", env!("CARGO_PKG_VERSION"));
}
