use ryguessr::name_gen::NameGenerator;

fn main() {
    let generator = NameGenerator::new();
    for _ in 0..10 {
        println!("{}", generator.generate());
    }
}
