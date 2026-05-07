pub struct Altitude {
    pub comment: String,
    distance: Option<Distance>,
}

impl Altitude {
    pub fn alt_string(&self) -> Option<String> {
        self.distance
            .map(|d| format!("{} ft ({} m)", d.feet(), d.meters()))
    }

    pub fn feet(&self) -> Option<i32> {
        self.distance.map(|d| d.feet())
    }

    pub fn meters(&self) -> Option<i32> {
        self.distance.map(|d| d.meters())
    }
}

impl From<&str> for Altitude {
    fn from(value: &str) -> Self {
        let (feet, comment) = extract_altitude(value);

        Self {
            distance: feet.map(Distance::Feet),
            comment,
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
enum Distance {
    Feet(i32),
    Meters(i32),
}

impl Distance {
    fn feet(self) -> i32 {
        match self {
            Self::Feet(f) => f,
            Self::Meters(m) => (m as f64 * 3.28084) as i32,
        }
    }

    fn meters(self) -> i32 {
        match self {
            Self::Feet(f) => (f as f64 / 3.28084) as i32,
            Self::Meters(m) => m,
        }
    }
}

// Extracts /A=XXXXXX altitude (feet) from a comment string.
// Returns (altitude_ft, comment_with_tag_removed).
// Consumes all consecutive digits after /A= (some stations send >6).
fn extract_altitude(comment: &str) -> (Option<i32>, String) {
    if let Some(pos) = comment.find("/A=") {
        let after = &comment[pos + 3..];
        let digit_count = after.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_count >= 6 {
            if let Ok(feet) = after[..digit_count].parse::<i32>() {
                let before = comment[..pos].trim_end();
                let rest = comment[pos + 3 + digit_count..].trim_start();
                let cleaned = match (before.is_empty(), rest.is_empty()) {
                    (true, true) => String::new(),
                    (true, false) => rest.to_string(),
                    (false, true) => before.to_string(),
                    (false, false) => format!("{} {}", before, rest),
                };
                return (Some(feet), cleaned.trim().to_string());
            }
        }
    }
    (None, comment.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── extract_altitude ───────────────────────────────────────────────

    #[test]
    fn altitude_at_start_of_comment() {
        let (ft, comment) = extract_altitude("/A=000000West Vancouver");
        assert_eq!(ft, Some(0));
        assert_eq!(comment, "West Vancouver");
    }

    #[test]
    fn altitude_in_middle_of_comment() {
        let (ft, comment) = extract_altitude("hello/A=000150world");
        assert_eq!(ft, Some(150));
        assert_eq!(comment, "hello world");
    }

    #[test]
    fn altitude_only_no_surrounding_text() {
        let (ft, comment) = extract_altitude("/A=001000");
        assert_eq!(ft, Some(1000));
        assert_eq!(comment, "");
    }

    #[test]
    fn altitude_not_present() {
        let (ft, comment) = extract_altitude("no altitude here");
        assert_eq!(ft, None);
        assert_eq!(comment, "no altitude here");
    }

    #[test]
    fn altitude_eight_digit_nonstandard() {
        // Some stations (e.g. MMDVM/Pi-Star) emit 8 digits instead of 6.
        let (ft, comment) = extract_altitude("/A=00000070 MMDVM Voice 434.650MHz");
        assert_eq!(ft, Some(70));
        assert_eq!(comment, "MMDVM Voice 434.650MHz");
    }
}
