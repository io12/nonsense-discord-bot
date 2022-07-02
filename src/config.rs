use std::fmt;

pub struct Config {
    pub auto_post_enabled: bool,
    pub pinging_enabled: bool,
    pub freq: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            auto_post_enabled: true,
            pinging_enabled: true,
            freq: 100,
        }
    }
}

fn get_state_str(state: bool) -> &'static str {
    if state {
        "enabled"
    } else {
        "disabled"
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Automatic posting is {}\n\
             Pinging is {}\n\
             Post frequency = {}",
            get_state_str(self.auto_post_enabled),
            get_state_str(self.pinging_enabled),
            self.freq,
        )
    }
}
