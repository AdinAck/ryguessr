use ryguessr::colors::DistinctColors;

fn main() {
    let mut colors = DistinctColors::new();

    for _ in 0..20 {
        let member_color = colors.next_filtered(&[]);
        let hex: String = member_color.into();
        // Parse the hex color to get RGB values for terminal display
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap();
        let g = u8::from_str_radix(&hex[3..5], 16).unwrap();
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap();

        // ANSI 24-bit color: \x1b[48;2;R;G;Bm for background
        println!("\x1b[48;2;{r};{g};{b}m    \x1b[0m {hex}");
    }
}
