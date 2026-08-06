use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

pub fn parse_duration(input: &str) -> Option<u64> {
    let input = input.trim().to_lowercase();
    if input.is_empty() {
        return None;
    }

    let mut total: u64 = 0;
    let mut amount: u64 = 0;
    let mut has_unit = false;

    for character in input.chars() {
        if let Some(digit) = character.to_digit(10) {
            amount = amount.checked_mul(10)?.checked_add(u64::from(digit))?;
            continue;
        }
        let seconds = match character {
            's' => 1,
            'm' => 60,
            'h' => 3_600,
            'd' => 86_400,
            'w' => 604_800,
            _ => return None,
        };
        total = total.checked_add(amount.checked_mul(seconds)?)?;
        amount = 0;
        has_unit = true;
    }

    if amount != 0 || !has_unit {
        return None;
    }
    Some(total)
}

pub fn format_duration(seconds: u64) -> String {
    if seconds == 0 {
        return "0s".to_owned();
    }

    let units = [("d", 86_400_u64), ("h", 3_600), ("m", 60), ("s", 1)];
    let mut remaining = seconds;
    let mut parts = Vec::new();

    for (suffix, size) in units {
        let value = remaining / size;
        if value > 0 {
            parts.push(format!("{value}{suffix}"));
            remaining -= value * size;
        }
    }

    parts.join(" ")
}
