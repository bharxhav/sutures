use sutures::Seam;

#[derive(Seam)]
enum Bad {
    #[seam(bogus)]
    A,
}

fn main() {}
