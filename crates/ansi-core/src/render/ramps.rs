//! Character ramps ordered from empty/dark to dense/bright.

pub const PRESETS: &[(&str, &str)] = &[
    ("standard", " .:-=+*#%@"),
    ("detailed", " .'`^\",:;Il!i><~+_-?][}{1)(|\\/tjfrxnuvczXYUJCLQ0OZmwqpdbkhao*#MW&8%B@$"),
    ("simple", " .oO@"),
    ("blocks", " ░▒▓█"),
    ("dots", " ·•●"),
    ("lines", " -=≡"),
    ("binary", " 01"),
];

/// Accepts a preset name or a literal ramp string.
pub fn resolve(name_or_chars: &str) -> Vec<char> {
    let chars = PRESETS
        .iter()
        .find(|(name, _)| *name == name_or_chars)
        .map_or(name_or_chars, |(_, ramp)| ramp);
    if chars.chars().count() < 2 {
        return PRESETS[0].1.chars().collect();
    }
    chars.chars().collect()
}
