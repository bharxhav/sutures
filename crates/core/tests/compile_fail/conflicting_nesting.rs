use sutures::Seam;

#[derive(Seam)]
struct Inner {
    x: i32,
}

#[derive(Seam)]
struct Bad {
    #[seam(to_struct, to_enum)]
    field: Inner,
}

fn main() {}
