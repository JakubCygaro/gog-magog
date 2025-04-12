use validator::ValidationError;

pub fn validate_user_login(login: &str) -> Result<(), ValidationError> {
    if !login.is_ascii() || login.contains(char::is_whitespace) {
        Err(ValidationError::new("2137")
            .with_message("username contains whitespace or non-ascii characters".into()))
    } else {
        Ok(())
    }
}

pub fn validate_user_password(password: &str) -> Result<(), ValidationError> {
    if !password.is_ascii() {
        Err(ValidationError::new("2138")
            .with_message("login contains non-ascii characters".into()))
    } else {
        Ok(())
    }
}
