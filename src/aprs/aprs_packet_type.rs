use colored::Color;

#[derive(Clone, Copy)]
pub enum AprsPacketType {
    Data,
    Item,
    Message,
    MicE,
    Object,
    Position,
    Status,
    Telemetry,
    Weather,
}

impl AprsPacketType {
    pub fn color(&self) -> Color {
        match self {
            Self::Data => Color::White,
            Self::Item => Color::BrightGreen,
            Self::Message => Color::Blue,
            Self::MicE => Color::Green,
            Self::Object => Color::BrightGreen,
            Self::Position => Color::Green,
            Self::Status => Color::Magenta,
            Self::Telemetry => Color::Yellow,
            Self::Weather => Color::White,
        }
    }
}

impl std::fmt::Display for AprsPacketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Data => write!(f, "Data"),
            Self::Item => write!(f, "Item"),
            Self::Message => write!(f, "Message"),
            Self::MicE => write!(f, "MicE"),
            Self::Object => write!(f, "Object"),
            Self::Position => write!(f, "Position"),
            Self::Status => write!(f, "Status"),
            Self::Telemetry => write!(f, "Telemetry"),
            Self::Weather => write!(f, "Weather"),
        }
    }
}

impl From<char> for AprsPacketType {
    fn from(c: char) -> Self {
        match c {
            '!' | '=' => Self::Position,
            '/' | '@' => Self::Position,
            ':' => Self::Message,
            '>' => Self::Status,
            'T' => Self::Telemetry,
            '_' => Self::Weather,
            ';' => Self::Object,
            ')' => Self::Item,
            '`' | '\'' => Self::MicE,
            _ => Self::Data,
        }
    }
}
