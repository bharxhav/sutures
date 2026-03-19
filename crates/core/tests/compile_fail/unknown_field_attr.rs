use sutures::Seam;

#[derive(Seam)]
struct Bad {
    #[seam(bogus)]
    field: String,
}

fn main() {}
