pub fn format_capabilities(data: &str) -> String {
    let mut parts: Vec<String> = Vec::new();

    for token in data.split(',') {
        let token = token.trim();
        match token {
            "IGATE" => parts.push("iGate".into()),
            "DIGI" => parts.push("Digipeater".into()),
            "RELAY" => parts.push("Relay".into()),
            "WIDE" => parts.push("Wide".into()),
            "GATE" => parts.push("Gate".into()),
            "WX" => parts.push("Weather".into()),
            "TCPIP" => parts.push("TCP/IP".into()),
            "TCPXX" => parts.push("TCP/IP".into()),
            _ => {
                if let Some((key, val)) = token.split_once('=') {
                    let label = match key {
                        "MSG_CNT" => format!("Messages gated: {}", val),
                        "LOC_CNT" => format!("Local stations: {}", val),
                        _ => token.to_string(),
                    };
                    parts.push(label);
                } else if !token.is_empty() {
                    parts.push(token.to_string());
                }
            }
        }
    }

    parts.join("  ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_known_tokens() {
        let result = format_capabilities("IGATE,WX,DIGI");
        assert!(result.contains("iGate"), "got: {result}");
        assert!(result.contains("Weather"), "got: {result}");
        assert!(result.contains("Digipeater"), "got: {result}");
    }

    #[test]
    fn capabilities_key_value_tokens() {
        let result = format_capabilities("MSG_CNT=42,LOC_CNT=7");
        assert!(result.contains("Messages gated: 42"), "got: {result}");
        assert!(result.contains("Local stations: 7"), "got: {result}");
    }

    #[test]
    fn capabilities_unknown_token_passthrough() {
        let result = format_capabilities("SOMEFLAG");
        assert!(result.contains("SOMEFLAG"), "got: {result}");
    }
}
