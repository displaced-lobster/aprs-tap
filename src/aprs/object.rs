use super::{Altitude, Position, Symbol, Weather};

#[allow(dead_code)]
pub struct Object<'a> {
    live: bool,
    name: &'a str,
    position: Position<'a>,
    timestamp: &'a str,
}

impl<'a> Object<'a> {
    pub fn position(&self) -> &Position<'a> {
        &self.position
    }

    pub fn format(&self) -> String {
        let pos = format!("{}, {}", self.position.lat, self.position.lon);
        let state = if self.live { "" } else { " [killed]" };
        let type_tag = Symbol::try_from((self.position.sym_table, self.position.sym_code))
            .map(|l| format!(" [{}]", l))
            .unwrap_or_default();
        if self.position.sym_code == '_' {
            let wx = Weather::from(self.position.comment).format();
            if wx.is_empty() {
                format!("{}{}{}: {}", self.name, state, type_tag, pos)
            } else {
                format!("{}{}{}: {} | {}", self.name, state, type_tag, pos, wx)
            }
        } else {
            let alt = Altitude::from(self.position.comment);
            let alt_str = alt.alt_string();
            match (alt.comment.is_empty(), alt_str) {
                (true, None) => format!("{}{}{}: {}", self.name, state, type_tag, pos),
                (false, None) => {
                    format!(
                        "{}{}{}: {} — {}",
                        self.name, state, type_tag, pos, alt.comment
                    )
                }
                (true, Some(a)) => format!("{}{}{}: {} — {}", self.name, state, type_tag, pos, a),
                (false, Some(a)) => {
                    format!(
                        "{}{}{}: {} — {} | {}",
                        self.name, state, type_tag, pos, alt.comment, a
                    )
                }
            }
        }
    }
}

impl<'a> TryFrom<&'a str> for Object<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        // OBJECTNAM*DDHHMMzDDMM.hhN/DDDMM.hhWCS[comment]
        // 9-char name, 1-char live/killed, 7-char timestamp, then position

        if value.len() < 17 {
            return Err(());
        }

        let name = value[..9].trim();
        let live = value.as_bytes().get(9) == Some(&b'*');
        let timestamp = value[10..17].trim();
        let position = Position::try_from(&value[17..])?;

        Ok(Self {
            live,
            name,
            position,
            timestamp,
        })
    }
}
