use ryguessr::colors::DistinctColors;

fn main() {
    for member_color in DistinctColors::new().take(20) {
        let srgb = member_color.srgb;
        // ANSI 24-bit color: \x1b[48;2;R;G;Bm for background
        println!(
            "\x1b[48;2;{};{};{}m    \x1b[0m {srgb}",
            srgb.r, srgb.g, srgb.b
        );
    }
}
