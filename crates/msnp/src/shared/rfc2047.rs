use std::fmt::format;
use email_encoding::headers::writer::EmailWriter;

const ENCODING_START_PREFIX: &str = "=?utf-8?b?";

const ENCODING_START_PREFIX_MAJ: &str = "=?utf-8?B?";
const ENCODING_END_SUFFIX: &str = "?=";
pub fn encode(data: &str) -> String {
    let mut encoded = String::new();
    {
        let mut writer = EmailWriter::new(&mut encoded, 0, 0, false);
        email_encoding::headers::rfc2047::encode(&data, &mut writer).expect("To have enough space");
    }

    encoded.replace_range(..ENCODING_START_PREFIX.len(), ENCODING_START_PREFIX_MAJ);
    encoded
}

#[cfg(test)]
mod tests {
    use super::encode;
    #[test]
    fn test_mail_encoding() {
        let name = "Inky";
        let encoded = encode(&name);

        assert_eq!("=?utf-8?B?SW5reQ==?=", &encoded);

    }

}