pub trait TryFromXml : Sized {
    type Error;

    fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error>;
}

pub trait ToXml {
    fn to_xml(self: &Self) -> &str;
}