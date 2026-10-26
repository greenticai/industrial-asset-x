//! Advisory-only rule: a recommendation is advice for an engineer, never a
//! command to a PLC, SCADA system or setpoint.
//!
//! The list is deliberately small. Matching is by whole word (or whole phrase)
//! after lowercasing, so `inspect`, `review` and `check` never match, and
//! neither does a word that merely contains a keyword (`rewrite` is not
//! `write`, `settlement` is not `set point`).

/// Keywords and phrases that make an action a command rather than advice.
/// Multi-word entries match as consecutive words; `set point` and `setpoint`
/// are both listed so either spelling is caught.
pub const FORBIDDEN_PHRASES: &[&str] = &[
    "write",
    "set point",
    "setpoint",
    "override",
    "actuate",
    "trip",
    "open breaker",
    "close breaker",
    "command",
];

fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Returns the first forbidden phrase found in `action`, if any.
pub fn command_phrase(action: &str) -> Option<&'static str> {
    let action_words = words(action);
    FORBIDDEN_PHRASES.iter().copied().find(|phrase| {
        let needle = words(phrase);
        action_words
            .windows(needle.len())
            .any(|window| window == needle.as_slice())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advice_verbs_are_accepted() {
        for action in [
            "inspect cooling system",
            "review signal quality",
            "check the oil level",
            "Schedule an inspection of the tap changer",
        ] {
            assert_eq!(command_phrase(action), None, "{action}");
        }
    }

    #[test]
    fn commands_are_refused_case_insensitively() {
        for (action, phrase) in [
            ("write setpoint 80 to PLC", "write"),
            ("Raise the SetPoint", "setpoint"),
            ("adjust set point to 70", "set point"),
            ("OVERRIDE the interlock", "override"),
            ("actuate the fan", "actuate"),
            ("trip the transformer", "trip"),
            ("Open Breaker 4", "open breaker"),
            ("close  breaker 4", "close breaker"),
            ("send command to SCADA", "command"),
        ] {
            assert_eq!(command_phrase(action), Some(phrase), "{action}");
        }
    }

    #[test]
    fn a_keyword_inside_another_word_is_not_a_match() {
        for action in [
            "rewrite the report",
            "review tripwire alarm history",
            "recommend a check",
        ] {
            assert_eq!(command_phrase(action), None, "{action}");
        }
    }
}
