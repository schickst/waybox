use std::env;
use std::fs::*;
use std::io::*;

use smithay::wayland::seat::ModifiersState;
use smithay::wayland::seat::XkbConfig;

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


    /*
    pub fn matches_modifiers(&self, modifiers: u32) -> bool {
        let mod_keys: Vec<String> = self.key_bindings.iter()
                                                           .map(|f| f.mod_key.clone())
                                                           .collect();

        for mod_key in mod_keys {
            match mod_key.as_str() {
                "Logo" => return modifiers & (wlr_keyboard_modifier_WLR_MODIFIER_LOGO) != 0,
                "Alt" => return modifiers & (wlr_keyboard_modifier_WLR_MODIFIER_ALT) != 0,
                &_ => continue
            }
        }
        false
    }

    pub fn handle_keybinding(&self, sym: xkb_keysym_t) -> bool {
        #[allow(non_upper_case_globals)]
        match sym {
            XKB_KEY_F1 => return self.execute_binding("F1"),
            XKB_KEY_F2 => return self.execute_binding("F2"),
            XKB_KEY_Q => return self.execute_binding("Q"),
            _ => return false
        }
    }

    fn execute_binding(&self, key: &str) -> bool {
        let binding = self.key_bindings.iter().find(|x| x.key == key);

        match binding {
            Some(b) => self.execute_command(&b.command),
            _ => false
        }
    }

    fn execute_command(&self, command: &str) -> bool {
        let handle = Command::new(command)
            .spawn()
            .expect("Command failed to start");
        true
    }
    */
}
