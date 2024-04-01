pub trait ParseStr {
    type Error;
    fn try_parse_str(str: &str) -> Result<Self, Self::Error> where Self: Sized;

}