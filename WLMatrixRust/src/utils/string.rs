
pub fn encode_utf16<B>(buf: &mut Vec<u8>, s: &str)
where
    B: byteorder::ByteOrder,
{
    for c in s.encode_utf16() {
        buf.extend(std::iter::repeat(0x0).take(2));
        let s = buf.len() - 2;
        B::write_u16(&mut buf[s..], c);
    }
}

pub fn map_empty_string_to_option(value: String) -> Option<String> {
    if !value.is_empty() {Some(value)} else {None}
}