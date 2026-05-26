#[path = "../common/mod.rs"]
mod common;
#[path = "../generators/mod.rs"]
mod generators;
mod shared;

fn main() {
    let dhat_dir = shared::resolve_dhat_dir();
    shared::profile_micro_sequences(&dhat_dir);
}
