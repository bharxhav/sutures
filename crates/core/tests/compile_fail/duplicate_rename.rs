use sutures::Seam;

#[derive(Seam)]
struct Bad {
    #[seam(rename = "a", rename = "b")]
    field: String,
}

fn main() {}
