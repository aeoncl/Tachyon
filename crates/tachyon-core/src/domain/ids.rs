use std::fmt::Display;
use std::sync::Arc;

macro_rules! str_id {
    ($name:ident) => {
        #[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
        pub struct $name(Arc<str>);

        impl $name {
            pub fn new(s: impl AsRef<str>) -> Self { Self(Arc::from(s.as_ref())) }
            pub fn as_str(&self) -> &str { &self.0 }
        }
        impl From<&str> for $name   { fn from(s: &str) -> Self { Self(Arc::from(s)) } }
        impl From<String> for $name { fn from(s: String) -> Self { Self(Arc::from(s)) } }
        impl std::ops::Deref for $name { type Target = str; fn deref(&self) -> &str { &self.0 } }
        impl AsRef<str> for $name { fn as_ref(&self) -> &str { &self.0 } }
        impl std::borrow::Borrow<str> for $name { fn borrow(&self) -> &str { &self.0 } }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str(&self.0) }
        }
    };
}

str_id!(DeviceId);
str_id!(LoginId);
str_id!(UserId);
str_id!(ConversationId);
str_id!(MediaId);
str_id!(SessionId);
str_id!(MessageId);