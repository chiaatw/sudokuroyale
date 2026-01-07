const FILLED: &str =
    "|---------=========---------=========---------=========---------=========---------|";
const EMPTY: &str =
    "|                                                                                 |";

/// Prints a progress bar to the console.
pub fn show_progress(size: usize) {
    println!("{}{}", &FILLED[..size + 1], &EMPTY[size + 1..]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_start() {
        let result = progress_bar(0);
        assert_eq!(result.len(), FILLED.len());
        assert_eq!(&result[..1], &FILLED[..1]);
    }

    #[test]
    fn test_progress_bar_middle() {
        let size = 10;
        let result = progress_bar(size);
        assert_eq!(&result[..size + 1], &FILLED[..size + 1]);
        assert_eq!(&result[size + 1..], &EMPTY[size + 1..]);
    }

    #[test]
    fn test_progress_bar_full() {
        let size = FILLED.len() - 1;
        let result = progress_bar(size);
        assert_eq!(result, FILLED);
    }
}
