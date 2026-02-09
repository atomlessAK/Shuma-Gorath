use once_cell::sync::Lazy;
use percent_encoding::percent_decode_str;
use serde_json::Value;
use std::collections::HashSet;
use std::net::IpAddr;

pub const MAX_ADMIN_JSON_BYTES: usize = 64 * 1024;
pub const MAX_CDP_REPORT_BYTES: usize = 16 * 1024;
pub const MAX_POW_VERIFY_BYTES: usize = 8 * 1024;
pub const MAX_CHALLENGE_FORM_BYTES: usize = 8 * 1024;
pub const MAX_BAN_REASON_LEN: usize = 120;
pub const MAX_CHECK_NAME_LEN: usize = 32;
pub const MAX_NONCE_LEN: usize = 512;
pub const MAX_SEED_TOKEN_LEN: usize = 4096;

static ISO_ALPHA2: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX",
        "AZ", "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ",
        "BR", "BS", "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK",
        "CL", "CM", "CN", "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM",
        "DO", "DZ", "EC", "EE", "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR",
        "GA", "GB", "GD", "GE", "GF", "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS",
        "GT", "GU", "GW", "GY", "HK", "HM", "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN",
        "IO", "IQ", "IR", "IS", "IT", "JE", "JM", "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN",
        "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC", "LI", "LK", "LR", "LS", "LT", "LU", "LV",
        "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK", "ML", "MM", "MN", "MO", "MP", "MQ",
        "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA", "NC", "NE", "NF", "NG", "NI",
        "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG", "PH", "PK", "PL", "PM",
        "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW", "SA", "SB", "SC",
        "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "SS", "ST", "SV",
        "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO", "TR",
        "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
        "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW",
    ]
    .into_iter()
    .collect()
});

pub fn enforce_body_size(body: &[u8], max_bytes: usize) -> Result<(), &'static str> {
    if body.len() > max_bytes {
        return Err("Payload too large");
    }
    Ok(())
}

pub fn parse_json_body(body: &[u8], max_bytes: usize) -> Result<Value, &'static str> {
    enforce_body_size(body, max_bytes)?;
    serde_json::from_slice::<Value>(body).map_err(|_| "Invalid JSON")
}

pub fn normalize_country_code_iso(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.len() != 2 || !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let upper = trimmed.to_ascii_uppercase();
    if ISO_ALPHA2.contains(upper.as_str()) {
        Some(upper)
    } else {
        None
    }
}

pub fn parse_ip_addr(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed.parse::<IpAddr>().ok().map(|addr| addr.to_string())
}

pub fn sanitize_admin_reason(input: &str) -> Result<String, &'static str> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok("admin_ban".to_string());
    }
    if trimmed.len() > MAX_BAN_REASON_LEN {
        return Err("Reason too long");
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err("Reason contains invalid characters");
    }
    Ok(trimmed.to_string())
}

pub fn sanitize_check_name(input: &str) -> Option<String> {
    let lowered = input.trim().to_ascii_lowercase();
    if lowered.is_empty() || lowered.len() > MAX_CHECK_NAME_LEN {
        return None;
    }
    if lowered
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-' || c == ':')
    {
        Some(lowered)
    } else {
        None
    }
}

pub fn validate_nonce(nonce: &str) -> bool {
    if nonce.is_empty() || nonce.len() > MAX_NONCE_LEN {
        return false;
    }
    nonce.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '+' || c == '/' || c == '='
    })
}

pub fn validate_seed_token(seed: &str) -> bool {
    if seed.is_empty() || seed.len() > MAX_SEED_TOKEN_LEN {
        return false;
    }
    seed.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || c == '_'
            || c == '-'
            || c == '+'
            || c == '/'
            || c == '='
            || c == '.'
    })
}

pub fn query_param(query: &str, key: &str) -> Option<String> {
    query.split('&').find_map(|pair| {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next()?;
        if k != key {
            return None;
        }
        let raw = parts.next().unwrap_or("");
        Some(percent_decode_str(raw).decode_utf8_lossy().to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_iso_country_code() {
        assert_eq!(normalize_country_code_iso("us").as_deref(), Some("US"));
        assert_eq!(normalize_country_code_iso(" JP ").as_deref(), Some("JP"));
    }

    #[test]
    fn rejects_non_iso_country_code() {
        assert!(normalize_country_code_iso("ZZ").is_none());
        assert!(normalize_country_code_iso("U1").is_none());
        assert!(normalize_country_code_iso("USA").is_none());
    }

    #[test]
    fn parses_ip_addresses() {
        assert_eq!(parse_ip_addr("127.0.0.1").as_deref(), Some("127.0.0.1"));
        assert_eq!(
            parse_ip_addr(" 2001:db8::1 ").as_deref(),
            Some("2001:db8::1")
        );
        assert!(parse_ip_addr("not_an_ip").is_none());
    }

    #[test]
    fn sanitizes_admin_reason() {
        assert_eq!(sanitize_admin_reason(" manual ").unwrap(), "manual");
        assert_eq!(sanitize_admin_reason(" ").unwrap(), "admin_ban");
        assert!(sanitize_admin_reason("a\nreason").is_err());
    }

    #[test]
    fn validates_seed_and_nonce() {
        assert!(validate_seed_token("abc.def=="));
        assert!(!validate_seed_token(""));
        assert!(validate_nonce("abc_123-+/="));
        assert!(!validate_nonce("bad nonce"));
    }

    #[test]
    fn parses_query_param_with_percent_decoding() {
        let q = "ip=2001%3Adb8%3A%3A1&x=1";
        assert_eq!(query_param(q, "ip").as_deref(), Some("2001:db8::1"));
    }

    #[test]
    fn parse_json_body_enforces_size_limit() {
        let big = vec![b'a'; MAX_POW_VERIFY_BYTES + 1];
        let err = parse_json_body(&big, MAX_POW_VERIFY_BYTES).unwrap_err();
        assert_eq!(err, "Payload too large");
    }

    #[test]
    fn sanitize_check_name_rejects_invalid_chars() {
        assert_eq!(
            sanitize_check_name("webdriver").as_deref(),
            Some("webdriver")
        );
        assert!(sanitize_check_name("bad check").is_none());
        assert!(sanitize_check_name("DROP TABLE;").is_none());
    }
}
