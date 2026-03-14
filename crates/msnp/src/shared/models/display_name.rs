use crate::shared::models::url_encoded_string::UrlEncodedString;

pub type DisplayName = UrlEncodedString;

#[cfg(test)]
mod tests {
    use crate::shared::models::display_name::DisplayName;

    #[test]
    fn default_value_is_empty() {
        assert_eq!(DisplayName::default().value(), "")
    }
}