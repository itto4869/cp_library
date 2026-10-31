#[macro_export]
macro_rules! yes_no {
    ($b:expr) => {
        $crate::yes_no_custom!($b, "Yes", "No")
    };
}

#[macro_export]
macro_rules! yes_no_custom {
    ($b:expr, $yes:expr, $no:expr) => {
        println!("{}", if $b { $yes } else { $no })
    };
}

#[must_use]
pub const fn yes_no(b: bool) -> &'static str {
    yes_no_custom(b, "Yes", "No")
}

#[must_use]
pub const fn yes_no_custom<'a>(b: bool, yes: &'a str, no: &'a str) -> &'a str {
    if b {
        yes
    } else {
        no
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yes_no() {
        assert_eq!(yes_no(true), "Yes");
        assert_eq!(yes_no(false), "No");
    }

    #[test]
    fn test_yes_no_custom() {
        assert_eq!(yes_no_custom(true, "YES", "NO"), "YES");
        assert_eq!(yes_no_custom(false, "Possible", "Impossible"), "Impossible");
    }

    #[test]
    fn test_yes_no_macro() {
        crate::yes_no!(true);
        crate::yes_no!(false);
    }

    #[test]
    fn test_yes_no_custom_macro() {
        crate::yes_no_custom!(true, "YES", "NO");
        crate::yes_no_custom!(false, "Possible", "Impossible");
    }
}
