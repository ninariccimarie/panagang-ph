use phonenumber::{metadata::DATABASE, Mode, PhoneNumber};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedPhone {
    pub e164: String,
    pub country_code: String,
    pub national_number: String,
}

#[derive(Debug, Error)]
pub enum PhoneError {
    #[error("invalid country code: {0}")]
    InvalidCountryCode(String),
    #[error("invalid phone number")]
    InvalidNumber,
}

/// Normalize a phone number to E.164 using country code + national (or full) number.
pub fn normalize_phone(
    country_code: &str,
    phone_number: &str,
) -> Result<NormalizedPhone, PhoneError> {
    let cc_digits: String = country_code
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();
    if cc_digits.is_empty() {
        return Err(PhoneError::InvalidCountryCode(country_code.to_string()));
    }
    let cc_u16: u16 = cc_digits
        .parse()
        .map_err(|_| PhoneError::InvalidCountryCode(country_code.to_string()))?;

    // Ensure the calling code is known to libphonenumber metadata.
    if DATABASE
        .by_code(&cc_u16)
        .map(|m| m.is_empty())
        .unwrap_or(true)
    {
        return Err(PhoneError::InvalidCountryCode(country_code.to_string()));
    }

    let trimmed = phone_number.trim();
    if trimmed.is_empty() {
        return Err(PhoneError::InvalidNumber);
    }

    let parsed: PhoneNumber = if trimmed.starts_with('+') {
        phonenumber::parse(None, trimmed).map_err(|_| PhoneError::InvalidNumber)?
    } else {
        let national = trimmed.trim_start_matches('0');
        let international = format!("+{cc_digits}{national}");
        phonenumber::parse(None, &international).map_err(|_| PhoneError::InvalidNumber)?
    };

    if !parsed.is_valid() {
        return Err(PhoneError::InvalidNumber);
    }

    // Reject numbers whose inferred country calling code does not match input.
    if parsed.country().code() != cc_u16 {
        return Err(PhoneError::InvalidNumber);
    }

    let e164 = parsed.format().mode(Mode::E164).to_string();
    let national = parsed.national().value().to_string();

    Ok(NormalizedPhone {
        e164,
        country_code: cc_u16.to_string(),
        national_number: national,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_ph_mobile() {
        let phone = normalize_phone("+63", "9171234567").expect("valid");
        assert_eq!(phone.e164, "+639171234567");
        assert_eq!(phone.country_code, "63");
        assert_eq!(phone.national_number, "9171234567");
    }

    #[test]
    fn normalizes_with_leading_zero_national() {
        let phone = normalize_phone("63", "09171234567").expect("valid");
        assert_eq!(phone.e164, "+639171234567");
    }

    #[test]
    fn rejects_invalid_number() {
        assert!(normalize_phone("+63", "123").is_err());
    }

    #[test]
    fn rejects_empty_country() {
        assert!(matches!(
            normalize_phone("+", "9171234567"),
            Err(PhoneError::InvalidCountryCode(_))
        ));
    }
}
