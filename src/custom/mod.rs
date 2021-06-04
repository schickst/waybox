use std::env;
use std::fs::File;
use std::io::Read;

use serde_json::Value;
use slog::Logger;
use smithay::{wayland::seat::ModifiersState};
use xkbcommon::xkb::keysym_from_name;
use xkbcommon::xkb::Keysym;
use xkbcommon::xkb::KEYSYM_NO_FLAGS;

use self::config::Keyboard;
use self::config::{Bar, MenuEntry};

pub mod config;

/*
lazy_static! {
    pub static ref CONFIGURATION: Configuration = {     
        Configuration::new("./config.json")
    };
}
*/



#[derive(Clone, Debug)]
pub struct Configuration {
    pub keyboard: Keyboard,
    pub key_bindings: KeyBindings,
    pub menu: Vec<MenuEntry>,
    pub bar: Bar,
    log: Logger
}


impl<'a> Configuration {
    pub fn new(log: Logger) -> Self {
        Configuration {
            keyboard: Keyboard::new(),
            key_bindings: KeyBindings::new(),
            menu: Vec::new(),
            bar: Bar::new(""),
            log
        }
    }

    pub fn parse(file: &str, log: Logger) -> Configuration {
        let data = Configuration::read_file(file);
        let raw_config: Value =
            serde_json::from_str(&data).expect("Unable to parse configuration");

        let keyboard = Keyboard::from(raw_config["keyboard"].clone());

        let raw_key_bindings = raw_config["key_bindings"].as_array().expect("Unable to parse key_bindings");
        let mut key_bindings = KeyBindings::from(raw_key_bindings);

        let bar = Bar::from(raw_config["bar"].clone());

        Configuration {
            keyboard,
            key_bindings,
            menu: Vec::new(),
            bar,
            log
        }
    }

    fn read_file(path: &str) -> String {
        if let Ok(current_path) = env::current_dir() {
            println!("The current directory is {}", current_path.display());
        }

        let mut file = File::open(path).expect("File not found");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read file");
        contents
    }
}


#[derive(Debug, PartialEq)]
/// Possible results of a keyboard action
pub enum KeyAction {
    /// Quit the compositor
    Quit,
    /// Trigger a vt-switch
    VtSwitch(i32),
    /// run a command
    Run(String),
    /// Switch the current screen
    Screen(usize),
    /// Forward the key to the client
    Forward,
    /// Do nothing more
    None,
}




#[derive(PartialEq, Eq, Clone, Debug)]
pub struct KeyBinding {
    description: String,
    keys: Vec<Keysym>,
    modifiers: ModifiersState,
    command: String
}


impl KeyBinding {
    fn from(value: Value) -> Self {
        let description = value["description"].as_str().expect("msg");
        let keys = value["keys"].as_str().expect("msg");
        let modifiers = value["modifiers"].as_str().expect("msg");
        let command = value["command"].as_str().expect("msg");

        let binding = KeyBinding {
            description: String::from(description),
            keys: KeyBinding::parse_keysyms(keys),
            modifiers: KeyBinding::parse_modkeys(modifiers),
            command: String::from(command)
        };
        binding
    }


    fn parse_keysyms(data: &str) -> Vec<Keysym> {
        let tokens = data.split(" ");
        let mut keysyms = Vec::new();

        for token in tokens {
            let keysym = keysym_from_name(token, KEYSYM_NO_FLAGS);
            keysyms.push(keysym);
        }
        keysyms
    }

    fn parse_modkeys(data: &str) -> ModifiersState {
        let tokens = data.split(" ");
        let mut mod_keys = ModifiersState::default();

        for token in tokens {
            if token == "Crtl" {
                mod_keys.ctrl = true;
            } else if token == "Alt" {
                mod_keys.alt = true;
            } else if token == "Shift" {
                mod_keys.shift = true;
            } else if token == "CapsLock" {
                mod_keys.caps_lock = true;
            } else if token == "Logo" {
                mod_keys.logo = true;
            } else if token == "NumLock" {
                mod_keys.num_lock = true;
            } else {
                println!("Unknown ModKey {}", token);
            }
        }
        mod_keys
    }
}




#[derive(PartialEq, Eq, Clone, Debug)]
pub struct KeyBindings {
    bindings: Vec<KeyBinding>
}


impl KeyBindings {
    fn new() -> Self {
        KeyBindings { bindings: Vec::new() }
    }

    fn from(values: &Vec<Value>) -> Self {
        let mut key_bindings = KeyBindings::new();

        for value in values {
            let binding = KeyBinding::from(value.clone());
            key_bindings.bindings.push(binding);
        }
        key_bindings
    }

    fn add_keybinding(&mut self, key_binding: KeyBinding) {
        self.bindings.push(key_binding);
    }

    pub fn process_keyboard_shortcut(&self, modifiers: ModifiersState, keysym: Keysym) -> KeyAction {
        for binding in &self.bindings {
            if binding.modifiers == modifiers &&
               binding.keys.contains(&keysym) {
                   return KeyAction::Run(binding.command.clone());
               }
        }
        KeyAction::Forward
    }
}

