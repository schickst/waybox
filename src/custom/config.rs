use std::env;
use std::fs::*;
use std::io::*;
use std::process::Command;
use smithay::wayland::seat::XkbConfig;
use serde_json::Value;

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Bar {
    command: String,
}

impl Bar {
    pub fn new(cmd: &str) -> Self {
        Bar { command: String::from(cmd) }
    }

    pub fn from(value: Value) -> Self {
        let bar: Bar = serde_json::from_value(value).expect("Unable to parse configuration");
        bar
    }

    pub fn spawn(&self) {
        if let Err(e) = Command::new(&self.command).spawn() {
            println!("Failed to start bar with command '{}'", self.command);
            println!("Error: {:?}", e);
        }
    }
}



#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct Keyboard {
    pub layout: String,
    pub variant: String,
    pub model: String,
}

impl<'a> Keyboard {
    pub fn new() -> Self {
        Keyboard {
            layout: String::from(""),
            variant: String::from(""),
            model: String::from(""),
        }
    }

    pub fn from(value: Value) -> Self {
        let keyboard: Keyboard = serde_json::from_value(value).expect("Unable to parse configuration");
        keyboard
    }

    pub fn get_seat_xkbconfig(&'a self) -> XkbConfig<'a> {
        XkbConfig {
            model: &self.model,
            layout: &self.layout,
            variant: &self.variant,
            ..XkbConfig::default()
        }
    }
}


#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct MenuEntry {
    title: String,
    command: String,
}
