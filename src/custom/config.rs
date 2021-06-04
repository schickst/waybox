use std::env;
use std::fs::*;
use std::io::*;

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Bar {
    command: String,
}

impl Bar {
    pub fn new() -> Self {
        Bar { command: String::new() }
    }
    
}



#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct RawKeyboardConfig {
    pub layout: String,
    pub variant: String,
    pub model: String,
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct MenuEntry {
    title: String,
    command: String,
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct RawKeyBinding {
    pub description: String,
    pub key: String,
    pub mod_key: String,
    pub command: String,
}



#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct RawConfiguration {
    pub keyboard: RawKeyboardConfig,
    pub key_bindings: Vec<RawKeyBinding>,
    pub menu: Vec<MenuEntry>,
    pub bar: Bar,
}

impl RawConfiguration {
    pub fn from_file(file: &str) -> RawConfiguration {
        let data = RawConfiguration::read_file(file);
        let config: RawConfiguration =
            serde_json::from_str(&data).expect("Unable to read configuration");
        return config;
    }

    fn read_file(path: &str) -> String {
        if let Ok(current_path) = env::current_dir() {
            println!("The current directory is {}", current_path.display());
        }

        let mut file = File::open(path).expect("File not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read file");
        return contents;
    }
}
