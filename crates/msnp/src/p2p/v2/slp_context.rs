use std::{char::decode_utf16, fmt::Display};

use byteorder::{ByteOrder, LittleEndian};


pub trait SlpContext {
    fn from_slp_context(bytes: &[u8]) -> Option<Self> where Self: Sized;
}

#[derive(Debug)]
pub struct PreviewData {
    size: usize,
    filename: String,
}

impl PreviewData {

    pub fn new(size: usize, filename: String) -> PreviewData {
        return PreviewData {size, filename};
    }

    pub fn get_size(&self) -> usize {
        return self.size;
    }

    pub fn get_filename(&self) -> String {
        return self.filename.clone();
    }


    fn to_slp_context(&self) -> Vec<u8> {
        let mut result = vec![0; 574];

        //context_size
        LittleEndian::write_u32(&mut result[0..4], 574);

        //tf_type
        LittleEndian::write_u32(&mut result[4..8], 2);

        //fileSize
        LittleEndian::write_u32(&mut result[8..12], self.size as u32);

        //Zero separator
        LittleEndian::write_u32(&mut result[12..16], 0);


        //Preview
        LittleEndian::write_u32(&mut result[16..20], 0);

        let mut test : Vec<u8> = Vec::new();

        let mut test_str = self.filename.clone();
        test_str.push('\0');

        encode_utf16::<LittleEndian>(&mut test, test_str.as_str());


        let slice = &mut result[20..test.len()+20];
        slice.clone_from_slice(test.as_slice());

        return result;
    }
}


fn encode_utf16<B>(buf: &mut Vec<u8>, s: &str)
where
    B: byteorder::ByteOrder,
{
    for c in s.encode_utf16() {
        buf.extend(std::iter::repeat(0x0).take(2));
        let s = buf.len() - 2;
        B::write_u16(&mut buf[s..], c);
    }
}


impl SlpContext for PreviewData {

    fn from_slp_context(bytes: &[u8]) -> Option<Self> {

        if bytes.len() >= 4 {
            let context_size = LittleEndian::read_u32(&bytes[0..4]) as usize;

            if context_size == 574 && bytes.len() >= context_size as usize {
                let tf_type = LittleEndian::read_u32(&bytes[4..8]);
                let file_size = LittleEndian::read_u32(&bytes[8..12]) as usize;
                let zero_separator = LittleEndian::read_u32(&bytes[12..16]);

                if zero_separator == 0 {
                    let has_preview = LittleEndian::read_u32(&bytes[16..20]) == 0;
                    //handle preview later
                    let filename_chunks: Vec<u16> = bytes[20..context_size].to_vec()
                    .chunks_exact(2)
                    .into_iter()
                    .map(|a| u16::from_le_bytes([a[0], a[1]]))
                    .collect();

                    let filename = decode_utf16(filename_chunks.into_iter()).map(|r| r.unwrap_or('�')).collect::<String>().trim_end_matches('\0').to_string();
                    return Some(PreviewData {size: file_size, filename});
                }
            }
        }

        return None;
    }
}

impl Display for PreviewData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base64 = base64::encode(self.to_slp_context());
        return write!(f, "{}", &base64);
    }
}

#[cfg(test)]
mod tests {
    use super::{PreviewData, SlpContext};

    #[test]
    fn preview_data_deserialization_test() {
        let base64_context = String::from("PgIAAAIAAACHPj4DAAAAAAEAAABnAGgAbwBzAHQALgBwAHMAZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==");
        let decoded = base64::decode(base64_context).unwrap();


        let result = PreviewData::from_slp_context(&decoded).unwrap();
        assert_eq!(result.get_filename(), String::from("ghost.psd"));
        assert_eq!(result.get_size(), 54410887);

    }

    #[test]
    fn preview_data_serialization_test() {
        let expected = String::from("PgIAAAIAAACHPj4DAAAAAAEAAABnAGgAbwBzAHQALgBwAHMAZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==");

        let preview_data = PreviewData::new(54410887, String::from("ghost.psd"));
        let result = preview_data.to_slp_context();



        let deserialized = PreviewData::from_slp_context(&result);

        println!("{:?}", deserialized);

        let base64 = base64::encode(result);

        assert_eq!(base64, expected);

    }


}