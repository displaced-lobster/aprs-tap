pub struct Capabilities<'a>(&'a str);

impl<'a> Capabilities<'a> {
    pub fn new(s: &'a str) -> Self {
        Self(s)
    }

    pub fn format(&self) -> String {
        self.0
            .split(',')
            .map(|s| match s.trim() {
                "IGATE" => "iGate".into(),
                "DIGI" => "Digipeater".into(),
                "RELAY" => "Relay".into(),
                "WIDE" => "Wide".into(),
                "GATE" => "Gate".into(),
                "WX" => "Weather".into(),
                "TCPIP" | "TCPXX" => "TCP/IP".into(),
                _ => {
                    if let Some((key, value)) = s.split_once('=') {
                        match key {
                            "MSG_CNT" => format!("Messages gated: {value}"),
                            "LOC_CNT" => format!("Local stations: {value}"),
                            _ => s.trim().to_string(),
                        }
                    } else {
                        s.trim().to_string()
                    }
                }
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_known_tokens() {
        let result = Capabilities::new("IGATE,WX,DIGI").format();
        assert!(result.contains("iGate"), "got: {result}");
        assert!(result.contains("Weather"), "got: {result}");
        assert!(result.contains("Digipeater"), "got: {result}");
    }

    #[test]
    fn capabilities_key_value_tokens() {
        let result = Capabilities::new("MSG_CNT=42,LOC_CNT=7").format();
        assert!(result.contains("Messages gated: 42"), "got: {result}");
        assert!(result.contains("Local stations: 7"), "got: {result}");
    }

    #[test]
    fn capabilities_unknown_token_passthrough() {
        let result = Capabilities::new("SOMEFLAG").format();
        assert!(result.contains("SOMEFLAG"), "got: {result}");
    }
}
