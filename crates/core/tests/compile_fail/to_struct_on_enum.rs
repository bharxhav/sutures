use sutures::Seam;

#[derive(Seam)]
enum Status {
    Active,
    Inactive,
}

#[derive(Seam)]
struct Bad {
    #[seam(to_struct)]
    status: Status,
}

fn main() {
    let _ = Bad::fields();
}
