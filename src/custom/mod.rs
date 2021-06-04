use lazy_static::lazy_static;
use smithay::wayland::seat::XkbConfig;
use smithay::{wayland::seat::ModifiersState};
use xkbcommon::xkb::keysym_from_name;
use xkbcommon::xkb::Keysym;
use xkbcommon::xkb::KEYSYM_NO_FLAGS;

use self::config::{Bar, MenuEntry, RawConfiguration, RawKeyBinding, RawKeyboardConfig};

pub mod config;


lazy_static! {
    pub static ref CONFIGURATION: Configuration = {     
        Configuration::new("./config.json")
    };
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct KeyBinding {
    description: String,
    key: Vec<Keysym>,
    mod_key: ModifiersState,
    command: String
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct KeyBindings {
    bindings: Vec<KeyBinding>
}


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


#[derive(Clone, Debug)]
pub struct Configuration {
    pub raw_config: RawConfiguration,
    pub key_bindings: KeyBindings,
    pub menu: Vec<MenuEntry>,
    pub bar: Bar,
}


impl<'a> Configuration {
    pub fn new(path: &str) -> Self {
        let raw_config = RawConfiguration::from_file(path);
        Configuration {
            raw_config: raw_config,
            key_bindings: KeyBindings::new(),
            menu: Vec::new(),
            bar: Bar::new()
        }
    }

    pub fn get_seat_xkbconfig(&'a self) -> XkbConfig<'a> {
        XkbConfig {
            model: &self.raw_config.keyboard.model,
            layout: &self.raw_config.keyboard.layout,
            variant: &self.raw_config.keyboard.variant,
            ..XkbConfig::default()
        }
    }
}



impl KeyBindings {
    fn new() -> Self {
        KeyBindings { bindings: Vec::new() }
    }

    fn add_keybinding(&mut self, config_keybinding: RawKeyBinding) {
        let binding = KeyBinding {
            description: config_keybinding.description.clone(),
            key: vec![ keysym_from_name(&config_keybinding.key, KEYSYM_NO_FLAGS) ],
            mod_key: ModifiersState {
                ctrl: true,
                alt: false,
                shift: false,
                caps_lock: false,
                logo: false,
                num_lock: false,
            },
            command: config_keybinding.command.clone()
        };
        self.bindings.push(binding);
    }

    fn parse_modkeys(&self, data: &str) -> ModifiersState {
        let tokens = data.split(" ");
        let mut mod_keys = ModifiersState::default();

        for token in tokens {
            if token == "Crtl" {
                mod_keys.ctrl = true;
            }
        }
        mod_keys
    }

    fn process_keyboard_shortcut(&self, modifiers: ModifiersState, keysym: Keysym) -> KeyAction {
        KeyAction::Forward
    }
}

