pub fn convert_base<T: std::fmt::Display>(num: T, n: u32, m: u32) -> Result<String, &'static str> {
    let s = num.to_string();
    let s = s.trim();
    if n < 2 || n > 36 || m < 2 || m > 36 {
        return Err("Bases must be between 2 and 36");
    }
    if s.is_empty() {
        return Err("Empty input");
    }

    let is_negative = s.starts_with('-');
    let mut s = if is_negative { &s[1..] } else { s };
    
    // Remove leading zeros
    while s.starts_with('0') && s.len() > 1 {
        s = &s[1..];
    }

    let mut digits = vec![];
    for c in s.chars() {
        let val = c.to_digit(n).ok_or("Invalid character for base n")?;
        digits.push(val);
    }

    if digits.is_empty() || digits.iter().all(|&d| d == 0) {
        return Ok("0".to_string());
    }

    let mut res = String::new();
    let chars_map = b"0123456789abcdefghijklmnopqrstuvwxyz";

    while !digits.is_empty() {
        let mut rem = 0;
        let mut next_digits = vec![];
        for &d in &digits {
            let cur = rem * n + d;
            let div = cur / m;
            rem = cur % m;
            if !next_digits.is_empty() || div > 0 {
                next_digits.push(div);
            }
        }
        res.push(chars_map[rem as usize] as char);
        digits = next_digits;
    }

    if is_negative && res != "0" {
        res.push('-');
    }

    Ok(res.chars().rev().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_base() {
        assert_eq!(convert_base("10", 10, 2).unwrap(), "1010");
        assert_eq!(convert_base("255", 10, 16).unwrap(), "ff");
        assert_eq!(convert_base("ff", 16, 10).unwrap(), "255");
        assert_eq!(convert_base(255, 10, 16).unwrap(), "ff");
        assert_eq!(convert_base(-10, 10, 2).unwrap(), "-1010");
        assert_eq!(convert_base("-ff", 16, 10).unwrap(), "-255");
        assert_eq!(convert_base("0", 10, 2).unwrap(), "0");
        assert_eq!(convert_base("000", 10, 2).unwrap(), "0");
        assert_eq!(convert_base(0, 16, 2).unwrap(), "0");
        
        let large_num = "123456789012345678901234567890";
        assert_eq!(convert_base(large_num, 10, 10).unwrap(), large_num);
    }
}
