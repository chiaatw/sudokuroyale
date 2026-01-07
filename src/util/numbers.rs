// Format numbers and durations

use std::time::Duration;

// Formats a duration in microseconds with commas
pub fn format_runtime(runtime: Duration) -> String {
    format_number(runtime.as_micros())
}

// Formats a number with commas as thousands separator
pub fn format_number(number: u128) -> String {
    number
        .to_string()
        .as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_format_number_small() {
        assert_eq!(format_number(0), "0");
        assert_eq!(format_number(5), "5");
        assert_eq!(format_number(12), "12");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_format_number_thousands() {
        assert_eq!(format_number(1234), "1,234");
        assert_eq!(format_number(12345), "12,345");
        assert_eq!(format_number(123456), "123,456");
    }

    #[test]
    fn test_format_number_millions() {
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(12345678), "12,345,678");
        assert_eq!(format_number(123456789), "123,456,789");
    }

    #[test]
    fn test_format_runtime() {
        let d1 = Duration::from_micros(0);
        let d2 = Duration::from_micros(1234);
        let d3 = Duration::from_micros(1_234_567);

        assert_eq!(format_runtime(d1), "0");
        assert_eq!(format_runtime(d2), "1,234");
        assert_eq!(format_runtime(d3), "1,234,567");
    }
}
