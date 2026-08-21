use std::fmt::Display;

pub trait ReduceToString {

    fn reduce_to_string(&self) -> String;

}

impl<T: Display> ReduceToString for &[T] {
    fn reduce_to_string(&self) -> String {
        if self.is_empty() {
            return "[]".to_string();
        }

        let mut out = String::new();
        for (index, current) in self.iter().enumerate() {
            out.push_str(&current.to_string());
            if index < self.len() {
                out.push_str(", ");
            }
        }

        out
    }
}

impl<T: Display> ReduceToString for Vec<T> {
    fn reduce_to_string(&self) -> String {
        self.as_slice().reduce_to_string()
    }
}