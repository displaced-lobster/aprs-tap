use colored::Colorize;

pub struct Message<'a> {
    pub addressee: &'a str,
    pub message: &'a str,
}

impl<'a> Message<'a> {
    pub fn format(&self) -> String {
        if self.addressee.is_empty() {
            self.message.to_string()
        } else {
            format!("{}: {}", self.addressee.bold(), self.message)
        }
    }
}

impl<'a> From<&'a str> for Message<'a> {
    // Format after DTI ':' is: "ADDRESSEE:message text{msgno}"
    // Addressee is exactly 9 chars (space-padded), followed by ':'
    fn from(value: &'a str) -> Self {
        if value.len() >= 10 && value.as_bytes()[9] == b':' {
            let addressee = value[..9].trim();
            let rest = &value[10..];
            let message = rest.rfind('{').map(|p| &rest[..p]).unwrap_or(rest).trim();

            Self { addressee, message }
        } else {
            Self {
                addressee: "",
                message: value,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_strips_sequence_number() {
        // data after DTI ':' is "ADDRESSEE:message{seqno}"
        let msg = Message::from("N0CALL   :Hello World{001}");
        let result = msg.format();
        assert!(result.contains("N0CALL"), "got: {result}");
        assert!(result.contains("Hello World"), "got: {result}");
        assert!(
            !result.contains("{001}"),
            "seq# should be stripped, got: {result}"
        );
    }

    #[test]
    fn message_no_sequence_number() {
        let msg = Message::from("VA7TEST  :Just a message");
        let result = msg.format();
        assert!(result.contains("VA7TEST"), "got: {result}");
        assert!(result.contains("Just a message"), "got: {result}");
    }

    #[test]
    fn message_bad_structure_passthrough() {
        let msg = Message::from("short:msg");
        let result = msg.format();
        assert_eq!(result, "short:msg");
    }
}
