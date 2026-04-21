use serde::{Deserialize, Serialize};

use crate::domain::errors::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum Priority {
    NoPriority,
    Urgent,
    High,
    Medium,
    Low,
}

impl TryFrom<u8> for Priority {
    type Error = DomainError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Priority::NoPriority),
            1 => Ok(Priority::Urgent),
            2 => Ok(Priority::High),
            3 => Ok(Priority::Medium),
            4 => Ok(Priority::Low),
            _ => Err(DomainError::InvalidInput(format!(
                "invalid priority value: {value}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_five_variants_exist() {
        let _ = Priority::NoPriority;
        let _ = Priority::Urgent;
        let _ = Priority::High;
        let _ = Priority::Medium;
        let _ = Priority::Low;
    }

    #[test]
    fn maps_linear_integers_0_to_4() {
        assert_eq!(Priority::try_from(0u8).unwrap(), Priority::NoPriority);
        assert_eq!(Priority::try_from(1u8).unwrap(), Priority::Urgent);
        assert_eq!(Priority::try_from(2u8).unwrap(), Priority::High);
        assert_eq!(Priority::try_from(3u8).unwrap(), Priority::Medium);
        assert_eq!(Priority::try_from(4u8).unwrap(), Priority::Low);
    }

    #[test]
    fn rejects_invalid_integer() {
        assert!(Priority::try_from(5u8).is_err());
    }

    #[test]
    fn round_trip_serde() {
        let p = Priority::High;
        let json = serde_json::to_string(&p).unwrap();
        let back: Priority = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Priority::High);
    }
}
