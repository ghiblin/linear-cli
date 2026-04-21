use is_terminal::IsTerminal;
use serde::Serialize;

pub fn format_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("serialization failed for well-typed value")
}

pub fn should_use_json(force_json: bool) -> bool {
    force_json || !std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Payload {
        key: &'static str,
    }

    #[test]
    fn format_json_produces_valid_json() {
        let output = format_json(&Payload { key: "value" });
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["key"], "value");
    }

    #[test]
    fn should_use_json_returns_true_when_forced() {
        assert!(should_use_json(true));
    }
}
