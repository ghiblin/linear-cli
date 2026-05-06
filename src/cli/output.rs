use is_terminal::IsTerminal;
use serde::Serialize;

pub fn format_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("serialization failed for well-typed value")
}

pub fn should_use_json(force_json: bool) -> bool {
    force_json || !std::io::stdout().is_terminal()
}

/// Returns true if JSON output should be used based on explicit flag arguments only.
/// Does not check TTY; callers wrap with `should_use_json(resolve_use_json(...))` for TTY fallback.
pub fn resolve_use_json(per_cmd_json: bool, output: Option<&str>, force_json: bool) -> bool {
    per_cmd_json || output == Some("json") || force_json
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

    #[test]
    fn resolve_use_json_per_cmd_flag_wins() {
        assert!(resolve_use_json(true, None, false));
    }

    #[test]
    fn resolve_use_json_output_json_wins() {
        assert!(resolve_use_json(false, Some("json"), false));
    }

    #[test]
    fn resolve_use_json_output_human_no_json() {
        assert!(!resolve_use_json(false, Some("human"), false));
    }

    #[test]
    fn resolve_use_json_per_cmd_beats_output_human() {
        assert!(resolve_use_json(true, Some("human"), false));
    }

    #[test]
    fn resolve_use_json_all_false() {
        assert!(!resolve_use_json(false, None, false));
    }

    #[test]
    fn resolve_use_json_force_json_wins() {
        assert!(resolve_use_json(false, None, true));
    }

    #[test]
    fn resolve_use_json_both_agree_json() {
        assert!(resolve_use_json(true, Some("json"), false));
    }
}
