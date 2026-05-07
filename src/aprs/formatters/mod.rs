mod capabilities;
mod message;
mod mice;
mod position;

pub(super) use capabilities::format_capabilities;
pub(super) use message::format_message;
pub(super) use mice::format_mice;
pub(super) use position::{format_object, format_position};

fn format_coord(deg: f64, pos: char, neg: char) -> String {
    let dir = if deg >= 0.0 { pos } else { neg };
    format!("{:.4}°{}", deg.abs(), dir)
}

#[cfg(test)]
mod tests {
    use super::*;

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
