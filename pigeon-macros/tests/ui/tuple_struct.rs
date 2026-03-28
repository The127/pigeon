use pigeon_macros::Reconstitute;

#[derive(Reconstitute)]
pub struct BadTuple(u64, String);

fn main() {}
