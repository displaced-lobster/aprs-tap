#[derive(Clone, Copy, Debug)]
pub struct Latitude(pub f64);

impl From<f64> for Latitude {
    fn from(value: f64) -> Self {
        Latitude(value)
    }
}

impl std::fmt::Display for Latitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format_coord(self.0, 'N', 'S'))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Longitude(pub f64);

impl From<f64> for Longitude {
    fn from(value: f64) -> Self {
        Longitude(value)
    }
}

impl std::fmt::Display for Longitude {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format_coord(self.0, 'E', 'W'))
    }
}

pub struct Position<'a> {
    pub lat: Latitude,
    pub lon: Longitude,
    pub sym_table: char,
    pub sym_code: char,
    pub comment: &'a str,
}

impl<'a> Position<'a> {
    // Parses APRS compressed position: SYYYYXXXXCS[comment]
    // S=sym_table, YYYY=lat(base91), XXXX=lon(base91), C=sym_code, csT=optional extension
    // Returns (lat, lon, sym_table, sym_code, comment)
    fn try_from_compressed(data: &'a str) -> Option<Self> {
        if data.len() < 10 {
            return None;
        }
        let b = data.as_bytes();

        // Bytes 1-8 (lat+lon) must be in printable base-91 range ('!' to '{', 33–123)
        for &byte in &b[1..9] {
            if !(33..=123).contains(&byte) {
                return None;
            }
        }

        let sym_table = b[0] as char;

        let lat_val = (b[1] - 33) as f64 * 753571.0   // 91^3
                    + (b[2] - 33) as f64 * 8281.0      // 91^2
                    + (b[3] - 33) as f64 * 91.0
                    + (b[4] - 33) as f64;
        let lat = 90.0 - lat_val / 380926.0;

        let lon_val = (b[5] - 33) as f64 * 753571.0
            + (b[6] - 33) as f64 * 8281.0
            + (b[7] - 33) as f64 * 91.0
            + (b[8] - 33) as f64;
        let lon = -180.0 + lon_val / 190463.0;

        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
            return None;
        }

        let sym_code = b[9] as char;
        // Bytes 10-12 are the optional csT (course/speed/compression-type) extension; comment follows
        let comment = if data.len() > 12 { &data[13..] } else { "" };

        Some(Self {
            lat: lat.into(),
            lon: lon.into(),
            sym_table,
            sym_code,
            comment,
        })
    }

    // Parses APRS uncompressed position: DDMM.hhN/DDDMM.hhWTS[comment]
    // Returns (lat, lon, sym_table, sym_code, comment)
    fn try_from_uncompressed(data: &'a str) -> Option<Self> {
        if data.len() < 19 {
            return None;
        }
        let b = data.as_bytes();
        let lat_dir = b[7] as char;
        let lon_dir = b[17] as char;

        if !matches!(lat_dir, 'N' | 'S') || !matches!(lon_dir, 'E' | 'W') {
            return None;
        }

        let lat_deg: f64 = data[0..2].parse().ok()?;
        let lat_min: f64 = data[2..7].parse().ok()?;
        let lon_deg: f64 = data[9..12].parse().ok()?;
        let lon_min: f64 = data[12..17].parse().ok()?;

        let lat = (lat_deg + lat_min / 60.0) * if lat_dir == 'S' { -1.0 } else { 1.0 };
        let lon = (lon_deg + lon_min / 60.0) * if lon_dir == 'W' { -1.0 } else { 1.0 };
        let sym_table = b[8] as char;
        let sym_code = b[18] as char;
        let comment = if data.len() > 19 { &data[19..] } else { "" };

        Some(Self {
            lat: lat.into(),
            lon: lon.into(),
            sym_table,
            sym_code,
            comment,
        })
    }
}

impl<'a> TryFrom<&'a str> for Position<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        // Uncompressed positions always have a decimal point at byte 4 (DDMM.hh).
        // Compressed positions use base-91 encoding with no such structure.
        if value.as_bytes().get(4) == Some(&b'.') {
            Self::try_from_uncompressed(value).ok_or(())
        } else {
            Self::try_from_compressed(value).ok_or(())
        }
    }
}

fn format_coord(deg: f64, pos: char, neg: char) -> String {
    let dir = if deg >= 0.0 { pos } else { neg };
    format!("{:.4}°{}", deg.abs(), dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    // ── parse_uncompressed_pos ──────────────────────────────────────────

    #[test]
    fn uncompressed_pos_north_west() {
        // 4854.41N/12332.35W_ (VA7ASI-WX weather station)
        let p = Position::try_from("4854.41N/12332.35W_012/000g003t055").unwrap();
        assert!(approx(p.lat.0, 48.9068), "lat={:?}", p.lat);
        assert!(approx(p.lon.0, -123.5392), "lon={:?}", p.lon);
        assert_eq!(p.sym_table, '/');
        assert_eq!(p.sym_code, '_');
        assert!(p.comment.starts_with("012/000"));
    }

    #[test]
    fn uncompressed_pos_south_west() {
        let p = Position::try_from("3322.50S/07030.00W>hello").unwrap();
        assert!(approx(p.lat.0, -(33.0 + 22.5 / 60.0)), "lat={:?}", p.lat);
        assert!(approx(p.lon.0, -(70.0 + 30.0 / 60.0)), "lon={:?}", p.lon);
    }

    #[test]
    fn uncompressed_pos_east_longitude() {
        // 51°30'N, 0°15'E
        let p = Position::try_from("5130.00N/00015.00E>").unwrap();
        assert!(approx(p.lat.0, 51.5), "lat={:?}", p.lat);
        assert!(approx(p.lon.0, 0.25), "lon={:?}", p.lon);
    }

    #[test]
    fn uncompressed_pos_extracts_comment() {
        let p = Position::try_from("4854.41N/12332.35W>Hello there").unwrap();
        assert_eq!(p.comment, "Hello there");
    }

    #[test]
    fn uncompressed_pos_empty_comment() {
        let p = Position::try_from("4854.41N/12332.35W>").unwrap();
        assert_eq!(p.comment, "");
    }

    #[test]
    fn uncompressed_pos_too_short_returns_none() {
        assert!(Position::try_from("4854.41N/123").is_err());
    }

    #[test]
    fn uncompressed_pos_bad_lat_dir_returns_none() {
        assert!(Position::try_from("4854.41X/12332.35W>").is_err());
    }

    #[test]
    fn uncompressed_pos_bad_lon_dir_returns_none() {
        assert!(Position::try_from("4854.41N/12332.35X>").is_err());
    }

    // ── parse_compressed_pos ───────────────────────────────────────────
    //
    // "/5c!!/F!!-" decodes to exactly lat=49.0°N, lon=-123.0°W
    // Derivation:
    //   lat_val = 20*753571 + 66*8281 + 0*91 + 0 = 15617966 → lat = 90 - 41.0 = 49.0
    //   lon_val = 14*753571 + 37*8281 + 0*91 + 0 = 10856391 → lon = -180 + 57.0 = -123.0
    const COMPRESSED_49N_123W: &str = "/5c!!/F!!-";

    #[test]
    fn compressed_pos_known_position() {
        let p = Position::try_from(COMPRESSED_49N_123W).unwrap();
        assert!(approx(p.lat.0, 49.0), "lat={:?}", p.lat);
        assert!(approx(p.lon.0, -123.0), "lon={:?}", p.lon);
        assert_eq!(p.sym_table, '/');
        assert_eq!(p.sym_code, '-');
        assert_eq!(p.comment, "");
    }

    #[test]
    fn compressed_pos_with_comment() {
        // 3 bytes of csT extension occupy positions 10-12; comment starts at 13
        let s = format!("{}abchello", COMPRESSED_49N_123W);
        let p = Position::try_from(s.as_str()).unwrap();
        assert_eq!(p.comment, "hello");
    }

    #[test]
    fn compressed_pos_too_short_returns_err() {
        assert!(Position::try_from("/5c!!/F!!").is_err()); // 9 chars, needs 10
    }

    #[test]
    fn compressed_pos_low_byte_returns_err() {
        let mut b = COMPRESSED_49N_123W.as_bytes().to_vec();
        b[3] = 0x20; // space = 32 < 33
        let s = String::from_utf8_lossy(&b).into_owned();
        assert!(Position::try_from(s.as_str()).is_err());
    }

    #[test]
    fn compressed_pos_high_byte_returns_err() {
        let mut b = COMPRESSED_49N_123W.as_bytes().to_vec();
        b[3] = 0x7C; // '|' = 124 > 123
        let s = String::from_utf8_lossy(&b).into_owned();
        assert!(Position::try_from(s.as_str()).is_err());
    }

    #[test]
    fn coord_north() {
        assert_eq!(format_coord(49.0, 'N', 'S'), "49.0000°N");
    }

    #[test]
    fn coord_south() {
        assert_eq!(format_coord(-33.5, 'N', 'S'), "33.5000°S");
    }

    #[test]
    fn coord_west() {
        assert_eq!(format_coord(-123.5, 'E', 'W'), "123.5000°W");
    }
}
