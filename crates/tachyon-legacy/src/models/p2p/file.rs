
#[derive(Clone, Debug)]
pub struct File {
    pub bytes: Vec<u8>,
    pub size: usize,
    pub filename: String
}

impl File {
    
    pub fn empty() -> Self {
        return File{bytes: Vec::new(), size: 0, filename: String::new()};
    }

    pub fn new(size: usize, filename: String) -> Self {
        return File{bytes: Vec::new(), size, filename};
    }

    pub fn get_mime(&self) -> String {
        let guess = new_mime_guess::from_path(self.filename.as_str());
        return guess.first_or_octet_stream().to_string();
    }

}



