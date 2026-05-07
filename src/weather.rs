use crate::direction::Direction;

#[derive(Default)]
pub struct Weather {
    wind_dir: Option<u16>,
    wind_speed: Option<u16>,
    wind_gust: Option<u16>,
    temp_f: Option<i32>,
    rain_1h: Option<f32>,
    rain_24h: Option<f32>,
    rain_midnight: Option<f32>,
    humidity: Option<u8>,
    pressure: Option<f32>,
}

impl Weather {
    pub fn format(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if let Some(t) = self.temp_f {
            let c = (t as f32 - 32.0) * 5.0 / 9.0;

            parts.push(format!("{:.1}°C ({t}°F)", c));
        }

        // APRS encodes 100% humidity as 00
        if let Some(h) = self.humidity {
            let pct = if h == 0 { 100u16 } else { h as u16 };
            parts.push(format!("{pct}% RH"));
        }

        match (self.wind_dir, self.wind_speed) {
            (Some(dir), Some(spd)) => {
                let compass = Direction::from(dir);
                let gust = self
                    .wind_gust
                    .filter(|&g| g > 0)
                    .map(|g| format!(" gusts {g} mph"))
                    .unwrap_or_default();
                if spd == 0 {
                    parts.push(format!("Calm ({compass}){gust}"));
                } else {
                    parts.push(format!("Wind: {compass} {spd} mph{gust}"));
                }
            }
            (None, Some(spd)) => parts.push(format!("Wind: {spd} mph")),
            _ => {}
        }

        if let Some(p) = self.pressure {
            parts.push(format!("{p:.1} hPa"));
        }

        if let Some(r) = self.rain_1h {
            if r > 0.0 {
                parts.push(format!("Rain 1h: {r:.2}\""));
            }
        }
        if let Some(r) = self.rain_24h {
            if r > 0.0 {
                parts.push(format!("Rain 24h: {r:.2}\""));
            }
        }
        if let Some(r) = self.rain_midnight {
            if r > 0.0 {
                parts.push(format!("Rain since midnight: {r:.2}\""));
            }
        }

        parts.join("  ")
    }
}

impl From<&str> for Weather {
    fn from(s: &str) -> Self {
        let mut w = Self::default();
        let b = s.as_bytes();

        // DDD/SSS wind at start of string
        if b.get(3) == Some(&b'/') {
            w.wind_dir = s.get(..3).and_then(|v| v.parse().ok());
            w.wind_speed = s.get(4..7).and_then(|v| v.parse().ok());
        }

        for i in 0..b.len() {
            match b[i] {
                b'c' if w.wind_dir.is_none() => {
                    w.wind_dir = s.get(i + 1..i + 4).and_then(|v| v.parse().ok());
                }
                b's' if w.wind_speed.is_none() => {
                    w.wind_speed = s.get(i + 1..i + 4).and_then(|v| v.parse().ok());
                }
                b'g' if w.wind_gust.is_none() => {
                    w.wind_gust = s.get(i + 1..i + 4).and_then(|v| v.parse().ok());
                }
                b't' if w.temp_f.is_none() => {
                    // Try lengths 4, 3, 2 to handle both tXXX and t-XX, t-XXX
                    w.temp_f = [4usize, 3, 2]
                        .iter()
                        .find_map(|&len| s.get(i + 1..i + 1 + len)?.parse().ok());
                }
                b'r' if w.rain_1h.is_none() => {
                    w.rain_1h = s
                        .get(i + 1..i + 4)
                        .and_then(|v| v.parse::<u32>().ok())
                        .map(|v| v as f32 / 100.0);
                }
                b'p' if w.rain_24h.is_none() => {
                    w.rain_24h = s
                        .get(i + 1..i + 4)
                        .and_then(|v| v.parse::<u32>().ok())
                        .map(|v| v as f32 / 100.0);
                }
                b'P' if w.rain_midnight.is_none() => {
                    w.rain_midnight = s
                        .get(i + 1..i + 4)
                        .and_then(|v| v.parse::<u32>().ok())
                        .map(|v| v as f32 / 100.0);
                }
                b'h' if w.humidity.is_none() => {
                    w.humidity = s.get(i + 1..i + 3).and_then(|v| v.parse().ok());
                }
                b'b' if w.pressure.is_none() => {
                    w.pressure = s
                        .get(i + 1..i + 6)
                        .and_then(|v| v.parse::<u32>().ok())
                        .map(|v| v as f32 / 10.0);
                }
                _ => {}
            }
        }

        w
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-3
    }

    // ── parse_weather ──────────────────────────────────────────────────

    #[test]
    fn weather_ddd_sss_prefix() {
        let w = Weather::from("270/015g022t072h65b10152");
        assert_eq!(w.wind_dir, Some(270));
        assert_eq!(w.wind_speed, Some(15));
        assert_eq!(w.wind_gust, Some(22));
        assert_eq!(w.temp_f, Some(72));
        assert_eq!(w.humidity, Some(65));
        assert!(
            approx(w.pressure.unwrap() as f64, 1015.2),
            "pressure={:?}",
            w.pressure
        );
    }

    #[test]
    fn weather_full_real_packet() {
        // VA7ASI-WX session packet weather data
        let w = Weather::from("012/000g003t055r000p000P000h99b10191");
        assert_eq!(w.wind_dir, Some(12));
        assert_eq!(w.wind_speed, Some(0));
        assert_eq!(w.wind_gust, Some(3));
        assert_eq!(w.temp_f, Some(55));
        assert_eq!(w.rain_1h, Some(0.0));
        assert_eq!(w.rain_24h, Some(0.0));
        assert_eq!(w.rain_midnight, Some(0.0));
        assert_eq!(w.humidity, Some(99));
        assert!(
            approx(w.pressure.unwrap() as f64, 1019.1),
            "pressure={:?}",
            w.pressure
        );
    }

    #[test]
    fn weather_negative_temp() {
        let w = Weather::from("t-24h50");
        assert_eq!(w.temp_f, Some(-24));
        assert_eq!(w.humidity, Some(50));
    }

    #[test]
    fn weather_empty_string() {
        let w = Weather::from("");
        assert!(w.wind_dir.is_none());
        assert!(w.temp_f.is_none());
        assert!(w.pressure.is_none());
    }

    // ── format_weather_line ────────────────────────────────────────────

    #[test]
    fn weather_line_humidity_zero_means_100_pct() {
        let w = Weather {
            humidity: Some(0),
            ..Default::default()
        };
        let s = w.format();
        assert!(s.contains("100% RH"), "got: {s}");
    }

    #[test]
    fn weather_line_temp_freezing() {
        let w = Weather {
            temp_f: Some(32),
            ..Default::default()
        };
        let s = w.format();
        assert!(s.contains("0.0°C"), "got: {s}");
        assert!(s.contains("32°F"), "got: {s}");
    }

    #[test]
    fn weather_line_wind_calm() {
        let w = Weather {
            wind_dir: Some(180),
            wind_speed: Some(0),
            ..Default::default()
        };
        let s = w.format();
        assert!(s.contains("Calm (S)"), "got: {s}");
    }

    #[test]
    fn weather_line_wind_with_speed_and_gust() {
        let w = Weather {
            wind_dir: Some(270),
            wind_speed: Some(15),
            wind_gust: Some(25),
            ..Default::default()
        };
        let s = w.format();
        assert!(s.contains("Wind: W 15 mph"), "got: {s}");
        assert!(s.contains("gusts 25 mph"), "got: {s}");
    }

    #[test]
    fn weather_line_zero_rain_not_shown() {
        let w = Weather {
            rain_1h: Some(0.0),
            rain_24h: Some(0.0),
            ..Default::default()
        };
        let s = w.format();
        assert!(!s.contains("Rain"), "got: {s}");
    }

    #[test]
    fn weather_line_nonzero_rain_shown() {
        let w = Weather {
            rain_1h: Some(0.25),
            ..Default::default()
        };
        let s = w.format();
        assert!(s.contains("Rain 1h: 0.25\""), "got: {s}");
    }
}
