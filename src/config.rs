use std::{
    collections::HashMap,
    env,
    ffi::OsString,
    fs, io,
    ops::{Add, AddAssign, MulAssign},
    os::windows::ffi::OsStringExt as _,
    path::PathBuf,
    str::FromStr,
    sync::LazyLock,
};

use windows::Win32::{Foundation::MAX_PATH, System::LibraryLoader::GetModuleFileNameW};

pub struct Config {
    pub enable_console: bool,
    pub keys_to_gamepad_map: Box<[(GamepadInput, i32)]>,
}

impl Config {
    fn from_file(file: String) -> Self {
        let map: HashMap<_, _> = file
            .split("\n")
            .filter_map(|kv_pair| {
                if let [key, value] = kv_pair.splitn(2, "=").collect::<Vec<_>>()[..] {
                    Some((key.trim().to_uppercase(), value.trim().to_uppercase()))
                } else {
                    None
                }
            })
            .collect();
        let enable_console = map
            .get("ENABLE_CONSOLE")
            .map(|value| matches!(value.as_ref(), "TRUE" | "1"))
            .unwrap_or(true);
        let keys_to_gamepad_map = map
            .into_iter()
            .filter_map(|(key, value)| {
                Some((
                    key.parse::<GamepadInput>().ok()?,
                    get_virtual_keycode(&value)?,
                ))
            })
            .collect();
        Self {
            enable_console,
            keys_to_gamepad_map,
        }
    }
}

fn get_virtual_keycode(key: &str) -> Option<i32> {
    match key {
        "VK_LBUTTON" => Some(0x01),
        "VK_RBUTTON" => Some(0x02),
        "VK_CANCEL" => Some(0x03),
        "VK_MBUTTON" => Some(0x04),
        "VK_XBUTTON1" => Some(0x05),
        "VK_XBUTTON2" => Some(0x06),
        "VK_BACK" => Some(0x08),
        "VK_TAB" => Some(0x09),
        "VK_CLEAR" => Some(0x0C),
        "VK_RETURN" => Some(0x0D),
        "VK_SHIFT" => Some(0x10),
        "VK_CONTROL" => Some(0x11),
        "VK_MENU" => Some(0x12),
        "VK_PAUSE" => Some(0x13),
        "VK_CAPITAL" => Some(0x14),
        "VK_KANA" => Some(0x15),
        "VK_HANGUL" => Some(0x15),
        "VK_IME_ON" => Some(0x16),
        "VK_JUNJA" => Some(0x17),
        "VK_FINAL" => Some(0x18),
        "VK_HANJA" => Some(0x19),
        "VK_KANJI" => Some(0x19),
        "VK_IME_OFF" => Some(0x1A),
        "VK_ESCAPE" => Some(0x1B),
        "VK_CONVERT" => Some(0x1C),
        "VK_NONCONVERT" => Some(0x1D),
        "VK_ACCEPT" => Some(0x1E),
        "VK_MODECHANGE" => Some(0x1F),
        "VK_SPACE" => Some(0x20),
        "VK_PRIOR" => Some(0x21),
        "VK_NEXT" => Some(0x22),
        "VK_END" => Some(0x23),
        "VK_HOME" => Some(0x24),
        "VK_LEFT" => Some(0x25),
        "VK_UP" => Some(0x26),
        "VK_RIGHT" => Some(0x27),
        "VK_DOWN" => Some(0x28),
        "VK_SELECT" => Some(0x29),
        "VK_PRINT" => Some(0x2A),
        "VK_EXECUTE" => Some(0x2B),
        "VK_SNAPSHOT" => Some(0x2C),
        "VK_INSERT" => Some(0x2D),
        "VK_DELETE" => Some(0x2E),
        "VK_HELP" => Some(0x2F),
        "0" => Some(0x30),
        "1" => Some(0x31),
        "2" => Some(0x32),
        "3" => Some(0x33),
        "4" => Some(0x34),
        "5" => Some(0x35),
        "6" => Some(0x36),
        "7" => Some(0x37),
        "8" => Some(0x38),
        "9" => Some(0x39),
        "A" => Some(0x41),
        "B" => Some(0x42),
        "C" => Some(0x43),
        "D" => Some(0x44),
        "E" => Some(0x45),
        "F" => Some(0x46),
        "G" => Some(0x47),
        "H" => Some(0x48),
        "I" => Some(0x49),
        "J" => Some(0x4A),
        "K" => Some(0x4B),
        "L" => Some(0x4C),
        "M" => Some(0x4D),
        "N" => Some(0x4E),
        "O" => Some(0x4F),
        "P" => Some(0x50),
        "Q" => Some(0x51),
        "R" => Some(0x52),
        "S" => Some(0x53),
        "T" => Some(0x54),
        "U" => Some(0x55),
        "V" => Some(0x56),
        "W" => Some(0x57),
        "X" => Some(0x58),
        "Y" => Some(0x59),
        "Z" => Some(0x5A),
        "VK_LWIN" => Some(0x5B),
        "VK_RWIN" => Some(0x5C),
        "VK_APPS" => Some(0x5D),
        "VK_SLEEP" => Some(0x5F),
        "VK_NUMPAD0" => Some(0x60),
        "VK_NUMPAD1" => Some(0x61),
        "VK_NUMPAD2" => Some(0x62),
        "VK_NUMPAD3" => Some(0x63),
        "VK_NUMPAD4" => Some(0x64),
        "VK_NUMPAD5" => Some(0x65),
        "VK_NUMPAD6" => Some(0x66),
        "VK_NUMPAD7" => Some(0x67),
        "VK_NUMPAD8" => Some(0x68),
        "VK_NUMPAD9" => Some(0x69),
        "VK_MULTIPLY" => Some(0x6A),
        "VK_ADD" => Some(0x6B),
        "VK_SEPARATOR" => Some(0x6C),
        "VK_SUBTRACT" => Some(0x6D),
        "VK_DECIMAL" => Some(0x6E),
        "VK_DIVIDE" => Some(0x6F),
        "VK_F1" => Some(0x70),
        "VK_F2" => Some(0x71),
        "VK_F3" => Some(0x72),
        "VK_F4" => Some(0x73),
        "VK_F5" => Some(0x74),
        "VK_F6" => Some(0x75),
        "VK_F7" => Some(0x76),
        "VK_F8" => Some(0x77),
        "VK_F9" => Some(0x78),
        "VK_F10" => Some(0x79),
        "VK_F11" => Some(0x7A),
        "VK_F12" => Some(0x7B),
        "VK_F13" => Some(0x7C),
        "VK_F14" => Some(0x7D),
        "VK_F15" => Some(0x7E),
        "VK_F16" => Some(0x7F),
        "VK_F17" => Some(0x80),
        "VK_F18" => Some(0x81),
        "VK_F19" => Some(0x82),
        "VK_F20" => Some(0x83),
        "VK_F21" => Some(0x84),
        "VK_F22" => Some(0x85),
        "VK_F23" => Some(0x86),
        "VK_F24" => Some(0x87),
        "VK_NUMLOCK" => Some(0x90),
        "VK_SCROLL" => Some(0x91),
        "VK_LSHIFT" => Some(0xA0),
        "VK_RSHIFT" => Some(0xA1),
        "VK_LCONTROL" => Some(0xA2),
        "VK_RCONTROL" => Some(0xA3),
        "VK_LMENU" => Some(0xA4),
        "VK_RMENU" => Some(0xA5),
        _ => None,
    }
}

#[derive(Debug)]
pub enum GamepadInput {
    Button(u16),
    LeftTrigger,
    RightTrigger,
    LeftStick(Vec2),
    RightStick(Vec2),
}

impl FromStr for GamepadInput {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DPAD_UP" => Ok(Self::Button(0x0001)),
            "DPAD_DOWN" => Ok(Self::Button(0x0002)),
            "DPAD_LEFT" => Ok(Self::Button(0x0004)),
            "DPAD_RIGHT" => Ok(Self::Button(0x0008)),
            "A" => Ok(Self::Button(0x1000)),
            "B" => Ok(Self::Button(0x2000)),
            "X" => Ok(Self::Button(0x4000)),
            "Y" => Ok(Self::Button(0x8000)),
            "START" => Ok(Self::Button(0x0010)),
            "BACK" => Ok(Self::Button(0x0020)),
            "LEFT_STICK_UP" => Ok(Self::LeftStick(Vec2::new(0., 1.))),
            "LEFT_STICK_DOWN" => Ok(Self::LeftStick(Vec2::new(0., -1.))),
            "LEFT_STICK_LEFT" => Ok(Self::LeftStick(Vec2::new(-1., 0.))),
            "LEFT_STICK_RIGHT" => Ok(Self::LeftStick(Vec2::new(1., 0.))),
            "LEFT_STICK_BUTTON" => Ok(Self::Button(0x0040)),
            "RIGHT_STICK_UP" => Ok(Self::RightStick(Vec2::new(0., 1.))),
            "RIGHT_STICK_DOWN" => Ok(Self::RightStick(Vec2::new(0., -1.))),
            "RIGHT_STICK_LEFT" => Ok(Self::RightStick(Vec2::new(-1., 0.))),
            "RIGHT_STICK_RIGHT" => Ok(Self::RightStick(Vec2::new(1., 0.))),
            "RIGHT_STICK_BUTTON" => Ok(Self::Button(0x0080)),
            "LEFT_SHOULDER" => Ok(Self::Button(0x0100)),
            "RIGHT_SHOULDER" => Ok(Self::Button(0x0200)),
            "LEFT_TRIGGER" => Ok(Self::LeftTrigger),
            "RIGHT_TRIGGER" => Ok(Self::RightTrigger),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2::new(0., 0.);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn magnitude(&self) -> f32 {
        f32::sqrt(self.x * self.x + self.y * self.y)
    }

    pub fn normalize(&mut self) {
        let magnitude = self.magnitude();
        self.x /= magnitude;
        self.y /= magnitude;
    }
}

impl AddAssign<&Self> for Vec2 {
    fn add_assign(&mut self, rhs: &Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl MulAssign<f32> for Vec2 {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

fn get_config() -> io::Result<Config> {
    let config_folder = [&env::var("AppData").unwrap_or_default(), "key2joy_rebinder"]
        .iter()
        .collect::<PathBuf>();
    fs::create_dir_all(&config_folder)?;
    let default_cfg_path = config_folder.join("default.cfg");
    let process_name = {
        let mut buf = [0; MAX_PATH as usize];
        let len = unsafe { GetModuleFileNameW(None, &mut buf) };
        PathBuf::from(OsString::from_wide(&buf[..len as usize]))
            .file_stem()
            .unwrap()
            .to_owned()
    };
    let cfg_path = config_folder.join(process_name).with_added_extension("cfg");
    if !fs::exists(&default_cfg_path)? {
        fs::write(&default_cfg_path, include_str!("default_config.cfg"))?;
    }
    if !fs::exists(&cfg_path)? {
        fs::copy(&default_cfg_path, &cfg_path)?;
    }
    fs::read_to_string(&cfg_path).map(Config::from_file)
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| get_config().unwrap());
