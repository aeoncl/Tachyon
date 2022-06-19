use std::char::decode_utf16;

use byteorder::{LittleEndian, ByteOrder};

pub trait SlpContext {
    fn from_slp_context(bytes: Vec<u8>) -> Option<Box<Self>>;
}


pub struct PreviewData {
    size: usize,
    filename: String,
}

impl PreviewData {
    pub fn get_size(&self) -> usize {
        return self.size;
    }

    pub fn get_filename(&self) -> String {
        return self.filename.clone();
    }
}

impl SlpContext for PreviewData {

    fn from_slp_context(bytes: Vec<u8>) -> Option<Box<Self>> { 

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
                    return Some(Box::new(PreviewData {size: file_size, filename: filename}));
                }
            }
        }

        return None;
    }
}

pub struct MsnObject {
    creator: String,
    size: usize,
    obj_type: i32,
    location: String,
    friendly: String,
    sha1d: Option<String>,
    sha1c: Option<String>
}

impl SlpContext for MsnObject {

    fn from_slp_context(bytes: Vec<u8>) -> Option<Box<Self>> { 
        return None;
    }
}



#[cfg(test)]
mod tests {
    use super::{PreviewData, SlpContext};


    #[test]
    fn preview_data_deserialization_test() {
        let base64_context = String::from("PgIAAAIAAACHPj4DAAAAAAEAAABnAGgAbwBzAHQALgBwAHMAZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==");
        let decoded = base64::decode(base64_context).unwrap();


        let result = PreviewData::from_slp_context(decoded).unwrap();
        assert_eq!(result.get_filename(), String::from("ghost.psd"));
        assert_eq!(result.get_size(), 54410887);

    }

    fn MsnObj_deserialization_test() {
        let base64_context = String::from("PG1zbm9iaiBDcmVhdG9yPSJidWRkeTFAaG90bWFpbC5jb20iIFNpemU9IjI0NTM5IiBUeXBlPSIzIiBMb2NhdGlvbj0iVEZSMkMudG1wIiBGcmllbmRseT0iQUFBPSIgU0hBMUQ9InRyQzhTbEZ4MnNXUXhaTUlCQVdTRW5YYzhvUT0iIFNIQTFDPSJVMzJvNmJvc1p6bHVKcTgyZUF0TXB4NWRJRUk9Ii8+DQoA");
    }

}