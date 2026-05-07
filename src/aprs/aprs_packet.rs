use colored::Colorize;

use super::{AprsPacketType, formatters::*};
use crate::weather::Weather;

pub struct AprsPacket<'a> {
    source: &'a str,
    destination: &'a str,
    path: Vec<&'a str>,
    info: &'a str,
    packet_type: AprsPacketType,
}

impl AprsPacket<'_> {
    pub fn display(&self, time: &str) {
        let path_str = if self.path.is_empty() {
            String::new()
        } else {
            format!(" via {}", self.path.join(","))
        };

        println!(
            "{} {} {} {}{}",
            format!("[{}]", time).dimmed(),
            self.source.cyan().bold(),
            "→".dimmed(),
            self.destination.yellow(),
            path_str.dimmed(),
        );
        println!(
            "           {} {}",
            format!("[{}]", self.packet_type).color(self.packet_type.color()),
            self.format_info(),
        );
        println!();
    }

    pub fn format_info(&self) -> String {
        let info = self.info;
        let Some(dti) = info.chars().next() else {
            return String::new();
        };
        let data = &info[dti.len_utf8()..];

        match dti {
            '!' | '=' => format_position(data),
            // Skip 7-char timestamp (DDHHMMz, HHMMSSh, etc.)
            '/' | '@' => format_position(if data.len() > 7 { &data[7..] } else { data }),
            ':' => format_message(data),
            ';' => format_object(data),
            // Positionless weather: skip 8-char timestamp (MMDDHHMMz)
            '_' => Weather::from(if data.len() > 8 { &data[8..] } else { data }).format(),
            '`' | '\'' => format_mice(self.destination, data),
            '<' => format_capabilities(data),
            _ => info.to_string(),
        }
    }
}

impl<'a> TryFrom<&'a str> for AprsPacket<'a> {
    type Error = ();

    fn try_from(line: &'a str) -> Result<Self, Self::Error> {
        let gt_pos = line.find('>').ok_or(())?;
        let source = &line[..gt_pos];
        let rest = &line[gt_pos + 1..];
        let colon_pos = rest.find(':').ok_or(())?;
        let path_part = &rest[..colon_pos];
        let info = &rest[colon_pos + 1..];
        let path_items: Vec<&str> = path_part.split(',').collect();
        let destination = path_items.first().ok_or(())?;
        let path = path_items[1..].to_vec();
        let dti = info.chars().next().unwrap_or(' ');
        let packet_type = AprsPacketType::from(dti);

        Ok(Self {
            source,
            destination,
            path,
            info,
            packet_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_aprs ──────────────────────────────────────────────────────

    #[test]
    fn parse_aprs_extracts_fields() {
        let pkt = AprsPacket::try_from("VA7ASI-1>APWW11,TCPIP*,qAC,T2CHILE:;hello").unwrap();
        assert_eq!(pkt.source, "VA7ASI-1");
        assert_eq!(pkt.destination, "APWW11");
        assert_eq!(pkt.path, vec!["TCPIP*", "qAC", "T2CHILE"]);
        assert_eq!(pkt.info, ";hello");
    }

    #[test]
    fn parse_aprs_no_path() {
        let pkt = AprsPacket::try_from("N0CALL>APRS:!data").unwrap();
        assert_eq!(pkt.source, "N0CALL");
        assert_eq!(pkt.destination, "APRS");
        assert!(pkt.path.is_empty());
        assert_eq!(pkt.info, "!data");
    }

    #[test]
    fn parse_aprs_no_gt_returns_none() {
        assert!(AprsPacket::try_from("NOCALLORCOLON").is_err());
    }

    #[test]
    fn parse_aprs_no_colon_returns_none() {
        assert!(AprsPacket::try_from("N0CALL>APRSnocorolon").is_err());
    }

    // ── format_info integration ─────────────────────────────────────────

    fn pkt<'a>(source: &'a str, dest: &'a str, info: &'a str) -> AprsPacket<'a> {
        AprsPacket {
            source,
            destination: dest,
            path: vec![],
            info,
            packet_type: AprsPacketType::from(info.chars().next().unwrap_or(' ')),
        }
    }

    #[test]
    fn format_info_uncompressed_position() {
        let p = pkt("VA7TEST", "APWW11", "!4854.41N/12332.35W>");
        let r = p.format_info();
        assert!(r.contains("48.9068°N"), "got: {r}");
        assert!(r.contains("123.5392°W"), "got: {r}");
        assert!(r.contains("Car"), "got: {r}");
    }

    #[test]
    fn format_info_weather_positionless() {
        // DTI '_' + 8-char timestamp + weather fields
        let p = pkt("VA7WX", "APWW11", "_01010000g003t055r000p000P000h99b10191");
        let r = p.format_info();
        // 55°F → 12.8°C
        assert!(r.contains("12.8°C"), "got: {r}");
    }

    #[test]
    fn format_info_capabilities() {
        let p = pkt("K7ABC-12", "APRS", "<IGATE,MSG_CNT=5");
        let r = p.format_info();
        assert!(r.contains("iGate"), "got: {r}");
        assert!(r.contains("Messages gated: 5"), "got: {r}");
    }

    #[test]
    fn format_info_message() {
        let p = pkt("VA7SRC", "APRS", ":VA7DEST  :Hello World{001}");
        let r = p.format_info();
        assert!(r.contains("VA7DEST"), "got: {r}");
        assert!(r.contains("Hello World"), "got: {r}");
    }
}
