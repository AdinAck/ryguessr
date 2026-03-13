pub struct NameGenerator {
    adjectives: Vec<&'static str>,
    nouns: Vec<&'static str>,
}

impl NameGenerator {
    pub fn new() -> Self {
        let adjectives = include_str!("../assets/english-adjectives.txt")
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        let nouns = include_str!("../assets/english-nouns.txt")
            .lines()
            .filter(|l| !l.trim().is_empty())
            .collect();
        Self { adjectives, nouns }
    }

    pub fn generate(&self) -> String {
        let adj = self.adjectives[rand::random_range(0..self.adjectives.len())];
        let noun = self.nouns[rand::random_range(0..self.nouns.len())];
        format!("{}-{}", adj, noun)
    }
}

impl Default for NameGenerator {
    fn default() -> Self {
        Self::new()
    }
}
