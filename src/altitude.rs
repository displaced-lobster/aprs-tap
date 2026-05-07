pub struct Altitude {
    pub feet: Option<i32>,
    pub comment: String,
}

impl Altitude {
    pub fn alt_string(&self) -> Option<String> {
        self.feet
            .map(|f| format!("{} ft ({} m)", f, self.meters().unwrap()))
    }

    pub fn meters(&self) -> Option<i32> {
        self.feet.map(|f| (f as f64 * 0.3048) as i32)
    }
}

impl From<&str> for Altitude {
    fn from(value: &str) -> Self {
        let (feet, comment) = extract_altitude(value);

        Self { feet, comment }
    }
}

// Extracts /A=XXXXXX altitude (feet) from a comment string.
// Returns (altitude_ft, comment_with_tag_removed).
fn extract_altitude(comment: &str) -> (Option<i32>, String) {
    if let Some(pos) = comment.find("/A=") {
        let after = &comment[pos + 3..];
        if after.len() >= 6 && after[..6].chars().all(|c| c.is_ascii_digit()) {
            if let Ok(feet) = after[..6].parse::<i32>() {
                let before = comment[..pos].trim_end();
                let rest = comment[pos + 9..].trim_start();
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
        // Real packet: "/A=000000West Vancouver"
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
}
