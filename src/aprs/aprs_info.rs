use colored::Color;

use super::{Capabilities, Message, MicE, Object, Position, Weather};

pub enum AprsInfo<'a> {
    Capabilities(Capabilities<'a>),
    Data(&'a str),
    Item(&'a str),
    Message(Message<'a>),
    MicE(MicE<'a>),
    Object(Object<'a>),
    Position(Position<'a>),
    Status(&'a str),
    Telemetry(&'a str),
    Weather {
        timestamp: Option<&'a str>,
        weather: Weather,
    },
}

impl<'a> AprsInfo<'a> {
    pub fn color(&self) -> Color {
        match self {
            Self::Capabilities(_) => Color::White,
            Self::Data(_) => Color::White,
            Self::Item(_) => Color::BrightGreen,
            Self::Message { .. } => Color::Blue,
            Self::MicE(_) => Color::Green,
            Self::Object(_) => Color::BrightGreen,
            Self::Position(_) => Color::Green,
            Self::Status(_) => Color::Magenta,
            Self::Telemetry(_) => Color::Yellow,
            Self::Weather { .. } => Color::White,
        }
    }

    pub fn format(&self) -> String {
        match self {
            Self::Capabilities(caps) => caps.format(),
            Self::Data(s) => s.to_string(),
            Self::Item(s) => s.to_string(),
            Self::Message(msg) => msg.format(),
            Self::MicE(mice) => mice.format(),
            Self::Object(obj) => obj.format(),
            Self::Position(pos) => pos.format(),
            Self::Status(s) => s.to_string(),
            Self::Telemetry(s) => s.to_string(),
            Self::Weather { weather, .. } => weather.format(),
        }
    }

    pub fn new(dest: &'a str, info: &'a str) -> Self {
        let dti = info.chars().next().unwrap_or(' ');
        let data = &info[dti.len_utf8()..];

        match dti {
            '<' => Self::Capabilities(Capabilities::new(data)),
            ')' => Self::Item(data),
            ':' => Self::Message(Message::from(data)),
            '`' | '\'' => MicE::try_new(dest, data)
                .map(Self::MicE)
                .unwrap_or(Self::Data(info)),
            ';' => Object::try_from(data)
                .map(Self::Object)
                .unwrap_or(Self::Data(info)),
            '!' | '=' => Position::try_from(data)
                .map(Self::Position)
                .unwrap_or(Self::Data(info)),
            '/' | '@' => {
                if data.len() > 7 {
                    Position::try_from(&data[7..])
                        .map(|p| Self::Position(p.with_timestamp(&data[..7])))
                        .unwrap_or(Self::Data(info))
                } else {
                    Position::try_from(data)
                        .map(Self::Position)
                        .unwrap_or(Self::Data(info))
                }
            }
            '>' => Self::Status(data),
            'T' => Self::Telemetry(data),
            '_' => {
                if data.len() > 8 {
                    Self::Weather {
                        timestamp: Some(&data[..8]),
                        weather: Weather::from(&data[8..]),
                    }
                } else {
                    Self::Weather {
                        timestamp: None,
                        weather: Weather::from(data),
                    }
                }
            }
            _ => Self::Data(data),
        }
    }

    pub fn type_str(&self) -> &str {
        match self {
            Self::Capabilities(_) => "Capabilities",
            Self::Data(_) => "Data",
            Self::Item(_) => "Item",
            Self::Message(_) => "Message",
            Self::MicE(_) => "MicE",
            Self::Object(_) => "Object",
            Self::Position(_) => "Position",
            Self::Status(_) => "Status",
            Self::Telemetry(_) => "Telemetry",
            Self::Weather { .. } => "Weather",
        }
    }
}
