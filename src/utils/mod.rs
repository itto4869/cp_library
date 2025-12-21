pub fn yes_no(b: bool) {
    println!("{}", if b { "Yes" } else { "No" });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yes_no() {
        // This just tests that it runs, capturing stdout is harder in simple tests
        yes_no(true);
        yes_no(false);
    }
}
