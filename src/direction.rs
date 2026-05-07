#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    N,
    NNE,
    NE,
    ENE,
    E,
    ESE,
    SE,
    SSE,
    S,
    SSW,
    SW,
    WSW,
    W,
    WNW,
    NW,
    NNW,
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

const DIRS: [Direction; 16] = [
    Direction::N,
    Direction::NNE,
    Direction::NE,
    Direction::ENE,
    Direction::E,
    Direction::ESE,
    Direction::SE,
    Direction::SSE,
    Direction::S,
    Direction::SSW,
    Direction::SW,
    Direction::WSW,
    Direction::W,
    Direction::WNW,
    Direction::NW,
    Direction::NNW,
];

impl From<u16> for Direction {
    fn from(value: u16) -> Self {
        DIRS[((value as f32 + 11.25) / 22.5) as usize % 16]
    }
}

impl std::str::FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u16>().map_err(|_| ()).map(Self::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compass_cardinal() {
        assert_eq!(Direction::from(0), Direction::N);
        assert_eq!(Direction::from(90), Direction::E);
        assert_eq!(Direction::from(180), Direction::S);
        assert_eq!(Direction::from(270), Direction::W);
    }

    #[test]
    fn compass_intercardinal() {
        assert_eq!(Direction::from(45), Direction::NE);
        assert_eq!(Direction::from(135), Direction::SE);
        assert_eq!(Direction::from(225), Direction::SW);
        assert_eq!(Direction::from(315), Direction::NW);
    }
}
