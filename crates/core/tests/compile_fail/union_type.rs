use sutures::Seam;

#[derive(Seam)]
union Bad {
    a: i32,
    b: f32,
}

fn main() {}
