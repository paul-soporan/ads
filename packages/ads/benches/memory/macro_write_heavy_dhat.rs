#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

fn main() {
    let dhat_dir = shared::resolve_dhat_dir();
    shared::profile_macro_write_heavy(&dhat_dir);
}
