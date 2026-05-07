use crate::{altitude::Altitude, position::Position, symbol::Symbol, weather::Weather};

pub fn format_object(data: &str) -> String {
    // OBJECTNAM*DDHHMMzDDMM.hhN/DDDMM.hhWCS[comment]
    // 9-char name, 1-char live/killed, 7-char timestamp, then position
    if data.len() < 17 {
        return data.to_string();
    }
    let name = data[..9].trim();
    let killed = data.as_bytes().get(9) == Some(&b'_');
    let pos_data = &data[17..]; // skip name(9) + indicator(1) + timestamp(7)

    match Position::try_from(pos_data) {
        Ok(Position {
            lat,
            lon,
            sym_table,
            sym_code,
            comment,
        }) => {
            let pos = format!("{lat}, {lon}");
            let state = if killed { " [killed]" } else { "" };
            let type_tag = Symbol::try_from((sym_table, sym_code))
                .map(|l| format!(" [{}]", l))
                .unwrap_or_default();
            if sym_code == '_' {
                let wx = Weather::from(comment).format();
                if wx.is_empty() {
                    format!("{}{}{}: {}", name, state, type_tag, pos)
                } else {
                    format!("{}{}{}: {} | {}", name, state, type_tag, pos, wx)
                }
            } else {
                let alt = Altitude::from(comment);
                let alt_str = alt.alt_string();
                match (alt.comment.is_empty(), alt_str) {
                    (true, None) => format!("{}{}{}: {}", name, state, type_tag, pos),
                    (false, None) => {
                        format!("{}{}{}: {} — {}", name, state, type_tag, pos, alt.comment)
                    }
                    (true, Some(a)) => format!("{}{}{}: {} — {}", name, state, type_tag, pos, a),
                    (false, Some(a)) => {
                        format!(
                            "{}{}{}: {} — {} | {}",
                            name, state, type_tag, pos, alt.comment, a
                        )
                    }
                }
            }
        }
        _ => data.to_string(),
    }
}

pub fn format_position(data: &str) -> String {
    match Position::try_from(data) {
        Ok(Position {
            lat,
            lon,
            sym_table,
            sym_code,
            comment,
        }) => {
            let pos = format!("{lat}, {lon}");
            if sym_code == '_' {
                let wx = Weather::from(comment).format();
                if wx.is_empty() {
                    pos
                } else {
                    format!("{} | {}", pos, wx)
                }
            } else {
                let alt = Altitude::from(comment);
                let label = Symbol::try_from((sym_table, sym_code));
                let alt_str = alt.alt_string();
                match (label, alt.comment.is_empty(), alt_str) {
                    (Ok(l), true, None) => format!("{} [{}]", pos, l),
                    (Ok(l), false, None) => format!("{} [{}] — {}", pos, l, alt.comment),
                    (Ok(l), true, Some(a)) => format!("{} [{}] — {}", pos, l, a),
                    (Ok(l), false, Some(a)) => format!("{} [{}] — {} | {}", pos, l, alt.comment, a),
                    (Err(_), true, None) => pos,
                    (Err(_), false, None) => format!("{} — {}", pos, alt.comment),
                    (Err(_), true, Some(a)) => format!("{} — {}", pos, a),
                    (Err(_), false, Some(a)) => format!("{} — {} | {}", pos, alt.comment, a),
                }
            }
        }
        _ => data.to_string(),
    }
}
