pub fn validate_username(username: &str) -> Result<(), &'static str> {
    let name = username.trim();

    if name.is_empty() {
        return Err("Username cannot be empty");
    }

    if name.len() < 3 || name.len() > 20 {
        return Err("Username must be at between 3 and 20 characters long");
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Username may only contain letter, numbers and underscores");
    }

    Ok(())
}

pub fn validate_email(email: &str) -> Result<(), &'static str> {
    let e = email.trim();

    if e.is_empty() {
        return Err("Email cannot be empty");
    }

    let pattern = regex::Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").expect("Invalid regex");

    if !pattern.is_match(e) {
        return Err("Email format is invalid");
    }

    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), &'static str> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long");
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter");
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one number");
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err("Password must contain at least one special character");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

   //username tests

    #[test]
    fn username_valid() {
        assert!(validate_username("Player_123").is_ok());
    }

    #[test]
    fn username_empty() {
        assert_eq!(
            validate_username("   "),
            Err("Username cannot be empty")
        );
    }

    #[test]
    fn username_too_short() {
        assert_eq!(
            validate_username("ab"),
            Err("Username must be at between 3 and 20 characters long")
        );
    }

    #[test]
    fn username_too_long() {
        let long_name = "a".repeat(21);
        assert_eq!(
            validate_username(&long_name),
            Err("Username must be at between 3 and 20 characters long")
        );
    }

    #[test]
    fn username_invalid_characters() {
        assert_eq!(
            validate_username("Player!"),
            Err("Username may only contain letter, numbers and underscores")
        );
    }

    //email tests

    #[test]
    fn email_valid() {
        assert!(validate_email("test@example.com").is_ok());
    }

    #[test]
    fn email_empty() {
        assert_eq!(
            validate_email(" "),
            Err("Email cannot be empty")
        );
    }

    #[test]
    fn email_invalid_format() {
        assert_eq!(
            validate_email("invalid-email"),
            Err("Email format is invalid")
        );
    }

    //passwort tests

    #[test]
    fn password_valid() {
        assert!(validate_password("Password1!").is_ok());
    }

    #[test]
    fn password_too_short() {
        assert_eq!(
            validate_password("Pw1!"),
            Err("Password must be at least 8 characters long")
        );
    }

    #[test]
    fn password_missing_uppercase() {
        assert_eq!(
            validate_password("password1!"),
            Err("Password must contain at least one uppercase letter")
        );
    }

    #[test]
    fn password_missing_number() {
        assert_eq!(
            validate_password("Password!"),
            Err("Password must contain at least one number")
        );
    }

    #[test]
    fn password_missing_special_char() {
        assert_eq!(
            validate_password("Password1"),
            Err("Password must contain at least one special character")
        );
    }
}