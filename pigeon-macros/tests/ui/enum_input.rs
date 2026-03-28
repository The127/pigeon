use pigeon_macros::Reconstitute;

#[derive(Reconstitute)]
pub enum BadEnum {
    A,
    B,
}

fn main() {}
