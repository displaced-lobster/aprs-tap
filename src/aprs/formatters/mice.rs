use crate::{
    direction::Direction,
    position::{Latitude, Longitude},
};

pub fn format_mice(dest: &str, data: &str) -> String {
    let db = dest.as_bytes();
    let ib = data.as_bytes();
    if db.len() < 6 || ib.len() < 8 {
        return data.to_string();
    }

    // Latitude from destination chars 0-5
    // Each char encodes one digit; chars 3-5 also encode N/S, lon offset, E/W
    let (d1, _) = mice_digit(db[0]);
    let (d2, _) = mice_digit(db[1]);
    let (d3, _) = mice_digit(db[2]);
    let (d4, north) = mice_digit(db[3]);
    let (d5, lon_offset) = mice_digit(db[4]);
    let (d6, west) = mice_digit(db[5]);

    let lat_deg = (d1 * 10 + d2) as f64;
    let lat_min = (d3 * 10 + d4) as f64 + (d5 * 10 + d6) as f64 / 100.0;
    let lat = Latitude::from((lat_deg + lat_min / 60.0) * if north { 1.0 } else { -1.0 });

    // Longitude from info bytes 0-2
    let mut lon_deg = ib[0] as f64 - 28.0;
    if lon_offset {
        lon_deg += 100.0;
    }
    if lon_deg >= 180.0 {
        lon_deg -= 80.0;
    }

    let mut lon_min = ib[1] as f64 - 28.0;
    if lon_min >= 60.0 {
        lon_min -= 60.0;
    }

    let lon_hund = ib[2] as f64 - 28.0;
    let lon = Longitude::from(
        (lon_deg + (lon_min + lon_hund / 100.0) / 60.0) * if west { -1.0 } else { 1.0 },
    );

    // Speed (knots) and course (degrees) from info bytes 3-5
    let sp = (ib[3] as i32 - 28) * 10;
    let dc = ib[4] as i32 - 28;
    let mut speed_kn = sp + dc / 10;
    if speed_kn >= 800 {
        speed_kn -= 800;
    }

    let mut course = (dc % 10) * 100 + (ib[5] as i32 - 28);
    if course >= 400 {
        course -= 400;
    }

    let pos = format!("{lat}, {lon}");
    let motion = if speed_kn == 0 {
        "Stationary".to_string()
    } else {
        format!(
            "{} kn @ {}° ({})",
            speed_kn,
            course,
            Direction::from(course as u16),
        )
    };

    let comment = if data.len() > 8 { data[8..].trim() } else { "" };
    if comment.is_empty() {
        format!("{} — {}", pos, motion)
    } else {
        format!("{} — {} — {}", pos, motion, comment)
    }
}

// Decodes one MicE destination character.
// Returns (digit_value, standard_flag).
// standard_flag = true → North / +100° lon offset / West (P-Z or 0-9 range)
// standard_flag = false → South / +0° lon offset / East (A-K range)
fn mice_digit(b: u8) -> (u8, bool) {
    match b as char {
        'A'..='J' => (b - b'A', false),
        'K' | 'L' => (0, b == b'L'),
        'P'..='Y' => (b - b'P', true),
        'Z' => (0, true),
        '0'..='9' => (b - b'0', true),
        _ => (0, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── mice_digit ─────────────────────────────────────────────────────

    #[test]
    fn mice_digit_numeric() {
        assert_eq!(mice_digit(b'0'), (0, true));
        assert_eq!(mice_digit(b'5'), (5, true));
        assert_eq!(mice_digit(b'9'), (9, true));
    }

    #[test]
    fn mice_digit_a_to_j() {
        assert_eq!(mice_digit(b'A'), (0, false));
        assert_eq!(mice_digit(b'J'), (9, false));
    }

    #[test]
    fn mice_digit_k_l() {
        assert_eq!(mice_digit(b'K'), (0, false));
        assert_eq!(mice_digit(b'L'), (0, true));
    }

    #[test]
    fn mice_digit_p_to_z() {
        assert_eq!(mice_digit(b'P'), (0, true));
        assert_eq!(mice_digit(b'Y'), (9, true));
        assert_eq!(mice_digit(b'Z'), (0, true));
    }

    // ── format_mice ────────────────────────────────────────────────────

    #[test]
    fn mice_decodes_real_packet() {
        // VA7ODR-7>EJBTUS...`4K8l E>/...
        // EJBTUS → lat≈49.2422°N, lon_offset=true, west=true
        // "4K8l E>/" → lon≈124.7880°W, speed=0 (Stationary)
        let result = format_mice("EJBTUS", "4K8l E>/extra comment");
        assert!(result.contains("49.2422°N"), "got: {result}");
        assert!(result.contains("124.7880°W"), "got: {result}");
        assert!(result.contains("Stationary"), "got: {result}");
        assert!(result.contains("extra comment"), "got: {result}");
    }

    #[test]
    fn mice_too_short_returns_raw() {
        // data < 8 bytes → falls back to returning data as-is
        let result = format_mice("EJBTUS", "short");
        assert_eq!(result, "short");
    }
}
