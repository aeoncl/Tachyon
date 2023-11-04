use std::env::Args;

pub struct SettingsLocator {
    pub homeserver_url : Option<String>
}


impl From<Args> for SettingsLocator {
    fn from(value: Args) -> Self {
        let mut out = SettingsLocator::new();
        out.init(value);
        return out;
    }
}
impl SettingsLocator {
    
    pub fn new() -> Self {
        Self{ homeserver_url: None }
    }

    pub fn init(self: &mut Self, value: Args) {
        let args : Vec<String> = value.collect();
        let mut current : Option<&str> = None;
        for arg in &args[1..args.len()] {
            if current.is_none() {
                current = Some(arg);
            }

            if current.is_some() {
                match current.unwrap() {
                    "--homeserverUrl" => {
                        self.homeserver_url = Some(arg.to_owned())
                    },
                    _ => {

                    }
                }
            }
        }
    }

}