use super::{
    Direction, Position,
    position::{Latitude, Longitude},
};

pub struct MicE<'a> {
    movement: Movement,
    position: Position<'a>,
}

impl<'a> MicE<'a> {
    // Decodes one MicE destination character.
    // Returns (digit_value, standard_flag).
    // standard_flag = true → North / +100° lon offset / West (P-Z or 0-9 range)
    // standard_flag = false → South / +0° lon offset / East (A-K range)
    fn digits(b: u8) -> (u8, bool) {
        match b as char {
            'A'..='J' => (b - b'A', false),
            'K' | 'L' => (0, b == b'L'),
            'P'..='Y' => (b - b'P', true),
            'Z' => (0, true),
            '0'..='9' => (b - b'0', true),
            _ => (0, false),
        }
    }

    pub fn position(&self) -> &Position<'a> {
        &self.position
    }

    pub fn format(&self) -> String {
        let pos_motion = format!(
            "{}, {} - {}",
            self.position.lat, self.position.lon, self.movement
        );

        if !self.position.comment.is_empty() {
            format!("{pos_motion} - {}", self.position.comment)
        } else {
            pos_motion
        }
    }

    pub fn try_new(dest: &'a str, data: &'a str) -> Result<Self, &'static str> {
        let db = dest.as_bytes();
        let ib = data.as_bytes();

        if db.len() < 6 || ib.len() < 8 {
            return Err("dest or data too short");
        }

        // Latitude from destination chars 0-5
        // Each char encodes one digit; chars 3-5 also encode N/S, lon offset, E/W
        let (d1, _) = Self::digits(db[0]);
        let (d2, _) = Self::digits(db[1]);
        let (d3, _) = Self::digits(db[2]);
        let (d4, north) = Self::digits(db[3]);
        let (d5, lon_offset) = Self::digits(db[4]);
        let (d6, west) = Self::digits(db[5]);

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
        let comment = if data.len() > 8 { data[8..].trim() } else { "" };
        let position = Position {
            comment,
            lat,
            lon,
            ..Default::default()
        };

        // Speed (knots) and course (degrees) from info bytes 3-5
        let sp = (ib[3] as i32 - 28) * 10;
        let dc = ib[4] as i32 - 28;
        let speed = Speed::new(sp + dc / 10);
        let course = Course::new((dc % 10) * 100 * (ib[5] as i32 - 28));
        let movement = Movement::new(course, speed);

        Ok(Self { movement, position })
    }
}

struct Course(i32);

impl Course {
    fn new(c: i32) -> Self {
        let mut course = c;

        if course >= 400 {
            course -= 400;
        }

        Self(course)
    }

    fn direction(&self) -> Direction {
        Direction::from(self.0 as u16)
    }
}

enum Movement {
    Stationary,
    Moving { course: Course, speed: Speed },
}

impl Movement {
    fn new(course: Course, speed: Speed) -> Self {
        if speed.0 == 0 {
            Self::Stationary
        } else {
            Self::Moving { course, speed }
        }
    }
}

impl std::fmt::Display for Movement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stationary => write!(f, "Stationary"),
            Self::Moving { course, speed } => {
                write!(f, "{} kn @ {}° ({})", speed.0, course.0, course.direction())
            }
        }
    }
}

struct Speed(i32);

impl Speed {
    fn new(speed_kn: i32) -> Self {
        let mut sp = speed_kn;

        if sp >= 800 {
            sp -= 800;
        }

        Self(sp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── mice_digit ─────────────────────────────────────────────────────

    #[test]
    fn mice_digit_numeric() {
        assert_eq!(MicE::digits(b'0'), (0, true));
        assert_eq!(MicE::digits(b'5'), (5, true));
        assert_eq!(MicE::digits(b'9'), (9, true));
    }

    #[test]
    fn mice_digit_a_to_j() {
        assert_eq!(MicE::digits(b'A'), (0, false));
        assert_eq!(MicE::digits(b'J'), (9, false));
    }

    #[test]
    fn mice_digit_k_l() {
        assert_eq!(MicE::digits(b'K'), (0, false));
        assert_eq!(MicE::digits(b'L'), (0, true));
    }

    #[test]
    fn mice_digit_p_to_z() {
        assert_eq!(MicE::digits(b'P'), (0, true));
        assert_eq!(MicE::digits(b'Y'), (9, true));
        assert_eq!(MicE::digits(b'Z'), (0, true));
    }

    // ── format_mice ────────────────────────────────────────────────────

    #[test]
    fn mice_decodes_real_packet() {
        // VA7ODR-7>EJBTUS...`4K8l E>/...
        // EJBTUS → lat≈49.2422°N, lon_offset=true, west=true
        // "4K8l E>/" → lon≈124.7880°W, speed=0 (Stationary)
        let result = MicE::try_new("EJBTUS", "4K8l E>/extra comment")
            .unwrap()
            .format();
        assert!(result.contains("49.2422°N"), "got: {result}");
        assert!(result.contains("124.7880°W"), "got: {result}");
        assert!(result.contains("Stationary"), "got: {result}");
        assert!(result.contains("extra comment"), "got: {result}");
    }

    #[test]
    fn mice_too_short_returns_err() {
        assert!(MicE::try_new("EJBTUS", "short").is_err());
    }
}
