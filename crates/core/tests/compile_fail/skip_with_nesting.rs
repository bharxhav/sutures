use sutures::Seam;

#[derive(Seam)]
struct Inner {
    x: i32,
}

#[derive(Seam)]
struct Bad {
    #[seam(skip, to_struct)]
    field: Inner,
}

fn main() {}
