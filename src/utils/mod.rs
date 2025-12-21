pub fn yes_no(b: bool) {
    yes_no_custom(b, "Yes", "No");
}

pub fn yes_no_custom(b: bool, yes: &str, no: &str) {
    println!("{}", if b { yes } else { no });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yes_no() {
        yes_no(true);
        yes_no(false);
    }

    #[test]
    fn test_yes_no_custom() {
        yes_no_custom(true, "YES", "NO");
        yes_no_custom(false, "Possible", "Impossible");
    }
}
