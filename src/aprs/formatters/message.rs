use colored::Colorize;

pub fn format_message(data: &str) -> String {
    // Format after DTI ':' is: "ADDRESSEE:message text{msgno}"
    // Addressee is exactly 9 chars (space-padded), followed by ':'
    if data.len() >= 10 && data.as_bytes()[9] == b':' {
        let addressee = data[..9].trim();
        let rest = &data[10..];
        let message = rest.rfind('{').map(|p| &rest[..p]).unwrap_or(rest).trim();
        format!("To {}: {}", addressee.bold(), message)
    } else {
        data.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_strips_sequence_number() {
        // data after DTI ':' is "ADDRESSEE:message{seqno}"
        let result = format_message("N0CALL   :Hello World{001}");
        assert!(result.contains("N0CALL"), "got: {result}");
        assert!(result.contains("Hello World"), "got: {result}");
        assert!(
            !result.contains("{001}"),
            "seq# should be stripped, got: {result}"
        );
    }

    #[test]
    fn message_no_sequence_number() {
        let result = format_message("VA7TEST  :Just a message");
        assert!(result.contains("VA7TEST"), "got: {result}");
        assert!(result.contains("Just a message"), "got: {result}");
    }

    #[test]
    fn message_bad_structure_passthrough() {
        let result = format_message("short:msg");
        assert_eq!(result, "short:msg");
    }
}
