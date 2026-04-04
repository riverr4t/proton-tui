use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Semantic color roles for the entire UI.
/// Using a theme struct allows easy swapping between color schemes
/// while keeping all color decisions in one place.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    // Base colors
    #[serde(with = "color_serde")]
    pub bg: Color,
    #[serde(with = "color_serde")]
    pub fg: Color,
    #[serde(with = "color_serde")]
    pub fg_dim: Color,
    #[serde(with = "color_serde")]
    pub fg_muted: Color,
    #[serde(with = "color_serde")]
    pub border: Color,
    #[serde(with = "color_serde")]
    pub border_active: Color,
    #[serde(with = "color_serde")]
    pub border_inactive: Color,

    // Highlight
    #[serde(with = "color_serde")]
    pub highlight_bg: Color,
    #[serde(with = "color_serde")]
    pub highlight_inactive_bg: Color,

    // Semantic
    #[serde(with = "color_serde")]
    pub accent: Color,
    #[serde(with = "color_serde")]
    pub success: Color,
    #[serde(with = "color_serde")]
    pub warning: Color,
    #[serde(with = "color_serde")]
    pub error: Color,
    #[serde(with = "color_serde")]
    pub info: Color,

    // Server load
    #[serde(with = "color_serde")]
    pub load_low: Color,
    #[serde(with = "color_serde")]
    pub load_medium: Color,
    #[serde(with = "color_serde")]
    pub load_high: Color,

    // UI elements
    #[serde(with = "color_serde")]
    pub search_active: Color,
    #[serde(with = "color_serde")]
    pub hint_key_fg: Color,
    #[serde(with = "color_serde")]
    pub hint_key_bg: Color,
    #[serde(with = "color_serde")]
    pub popup_bg: Color,
    #[serde(with = "color_serde")]
    pub secure_core: Color,

    // Help popup
    #[serde(with = "color_serde")]
    pub help_bg: Color,
    #[serde(with = "color_serde")]
    pub help_border: Color,
    #[serde(with = "color_serde")]
    pub help_accent: Color,
    #[serde(with = "color_serde")]
    pub help_section: Color,
    #[serde(with = "color_serde")]
    pub help_key_fg: Color,
    #[serde(with = "color_serde")]
    pub help_key_bg: Color,
    #[serde(with = "color_serde")]
    pub help_desc: Color,
    #[serde(with = "color_serde")]
    pub help_divider: Color,
    #[serde(with = "color_serde")]
    pub help_footer_bg: Color,
    #[serde(with = "color_serde")]
    pub help_footer_fg: Color,

    // Connection popup
    #[serde(with = "color_serde")]
    pub upload: Color,
}

/// Default theme matches the original hardcoded colors exactly (zero visual change).
impl Default for Theme {
    fn default() -> Self {
        Self {
            // Base
            bg: Color::Black,
            fg: Color::White,
            fg_dim: Color::Gray,
            fg_muted: Color::DarkGray,
            border: Color::Gray,
            border_active: Color::Green,
            border_inactive: Color::DarkGray,

            // Highlight
            highlight_bg: Color::Rgb(40, 44, 52),
            highlight_inactive_bg: Color::Rgb(30, 32, 36),

            // Semantic
            accent: Color::Green,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,

            // Server load
            load_low: Color::Green,
            load_medium: Color::Yellow,
            load_high: Color::Red,

            // UI elements
            search_active: Color::Yellow,
            hint_key_fg: Color::Black,
            hint_key_bg: Color::DarkGray,
            popup_bg: Color::Black,
            secure_core: Color::Magenta,

            // Help popup
            help_bg: Color::Rgb(22, 22, 30),
            help_border: Color::Rgb(88, 91, 112),
            help_accent: Color::Rgb(137, 180, 250),
            help_section: Color::Rgb(166, 227, 161),
            help_key_fg: Color::Rgb(249, 226, 175),
            help_key_bg: Color::Rgb(49, 50, 68),
            help_desc: Color::Rgb(205, 214, 244),
            help_divider: Color::Rgb(69, 71, 90),
            help_footer_bg: Color::Rgb(49, 50, 68),
            help_footer_fg: Color::Rgb(186, 194, 222),

            // Connection popup
            upload: Color::Magenta,
        }
    }
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        match name {
            "catppuccin_mocha" => Self::catppuccin_mocha(),
            "dracula" => Self::dracula(),
            "nord" => Self::nord(),
            "tokyo_night" => Self::tokyo_night(),
            _ => Self::default(),
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            bg: Color::Rgb(30, 30, 46),
            fg: Color::Rgb(205, 214, 244),
            fg_dim: Color::Rgb(166, 173, 200),
            fg_muted: Color::Rgb(108, 112, 134),
            border: Color::Rgb(88, 91, 112),
            border_active: Color::Rgb(137, 180, 250),
            border_inactive: Color::Rgb(69, 71, 90),

            highlight_bg: Color::Rgb(49, 50, 68),
            highlight_inactive_bg: Color::Rgb(39, 39, 55),

            accent: Color::Rgb(137, 180, 250),
            success: Color::Rgb(166, 227, 161),
            warning: Color::Rgb(249, 226, 175),
            error: Color::Rgb(243, 139, 168),
            info: Color::Rgb(137, 180, 250),

            load_low: Color::Rgb(166, 227, 161),
            load_medium: Color::Rgb(249, 226, 175),
            load_high: Color::Rgb(243, 139, 168),

            search_active: Color::Rgb(249, 226, 175),
            hint_key_fg: Color::Rgb(30, 30, 46),
            hint_key_bg: Color::Rgb(108, 112, 134),
            popup_bg: Color::Rgb(24, 24, 37),
            secure_core: Color::Rgb(203, 166, 247),

            help_bg: Color::Rgb(24, 24, 37),
            help_border: Color::Rgb(88, 91, 112),
            help_accent: Color::Rgb(137, 180, 250),
            help_section: Color::Rgb(166, 227, 161),
            help_key_fg: Color::Rgb(249, 226, 175),
            help_key_bg: Color::Rgb(49, 50, 68),
            help_desc: Color::Rgb(205, 214, 244),
            help_divider: Color::Rgb(69, 71, 90),
            help_footer_bg: Color::Rgb(49, 50, 68),
            help_footer_fg: Color::Rgb(186, 194, 222),

            upload: Color::Rgb(203, 166, 247),
        }
    }

    pub fn dracula() -> Self {
        Self {
            bg: Color::Rgb(40, 42, 54),
            fg: Color::Rgb(248, 248, 242),
            fg_dim: Color::Rgb(189, 147, 249),
            fg_muted: Color::Rgb(98, 114, 164),
            border: Color::Rgb(68, 71, 90),
            border_active: Color::Rgb(139, 233, 253),
            border_inactive: Color::Rgb(68, 71, 90),

            highlight_bg: Color::Rgb(68, 71, 90),
            highlight_inactive_bg: Color::Rgb(54, 56, 72),

            accent: Color::Rgb(139, 233, 253),
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(241, 250, 140),
            error: Color::Rgb(255, 85, 85),
            info: Color::Rgb(139, 233, 253),

            load_low: Color::Rgb(80, 250, 123),
            load_medium: Color::Rgb(241, 250, 140),
            load_high: Color::Rgb(255, 85, 85),

            search_active: Color::Rgb(241, 250, 140),
            hint_key_fg: Color::Rgb(40, 42, 54),
            hint_key_bg: Color::Rgb(98, 114, 164),
            popup_bg: Color::Rgb(33, 34, 44),
            secure_core: Color::Rgb(189, 147, 249),

            help_bg: Color::Rgb(33, 34, 44),
            help_border: Color::Rgb(68, 71, 90),
            help_accent: Color::Rgb(139, 233, 253),
            help_section: Color::Rgb(80, 250, 123),
            help_key_fg: Color::Rgb(241, 250, 140),
            help_key_bg: Color::Rgb(68, 71, 90),
            help_desc: Color::Rgb(248, 248, 242),
            help_divider: Color::Rgb(68, 71, 90),
            help_footer_bg: Color::Rgb(68, 71, 90),
            help_footer_fg: Color::Rgb(248, 248, 242),

            upload: Color::Rgb(189, 147, 249),
        }
    }

    pub fn nord() -> Self {
        Self {
            bg: Color::Rgb(46, 52, 64),
            fg: Color::Rgb(216, 222, 233),
            fg_dim: Color::Rgb(178, 190, 205),
            fg_muted: Color::Rgb(76, 86, 106),
            border: Color::Rgb(67, 76, 94),
            border_active: Color::Rgb(136, 192, 208),
            border_inactive: Color::Rgb(59, 66, 82),

            highlight_bg: Color::Rgb(59, 66, 82),
            highlight_inactive_bg: Color::Rgb(52, 59, 73),

            accent: Color::Rgb(136, 192, 208),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            info: Color::Rgb(129, 161, 193),

            load_low: Color::Rgb(163, 190, 140),
            load_medium: Color::Rgb(235, 203, 139),
            load_high: Color::Rgb(191, 97, 106),

            search_active: Color::Rgb(235, 203, 139),
            hint_key_fg: Color::Rgb(46, 52, 64),
            hint_key_bg: Color::Rgb(76, 86, 106),
            popup_bg: Color::Rgb(36, 40, 50),
            secure_core: Color::Rgb(180, 142, 173),

            help_bg: Color::Rgb(36, 40, 50),
            help_border: Color::Rgb(67, 76, 94),
            help_accent: Color::Rgb(136, 192, 208),
            help_section: Color::Rgb(163, 190, 140),
            help_key_fg: Color::Rgb(235, 203, 139),
            help_key_bg: Color::Rgb(59, 66, 82),
            help_desc: Color::Rgb(216, 222, 233),
            help_divider: Color::Rgb(59, 66, 82),
            help_footer_bg: Color::Rgb(59, 66, 82),
            help_footer_fg: Color::Rgb(216, 222, 233),

            upload: Color::Rgb(180, 142, 173),
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            bg: Color::Rgb(26, 27, 38),
            fg: Color::Rgb(192, 202, 245),
            fg_dim: Color::Rgb(145, 155, 200),
            fg_muted: Color::Rgb(68, 75, 106),
            border: Color::Rgb(56, 62, 90),
            border_active: Color::Rgb(125, 174, 247),
            border_inactive: Color::Rgb(41, 46, 66),

            highlight_bg: Color::Rgb(41, 46, 66),
            highlight_inactive_bg: Color::Rgb(33, 37, 52),

            accent: Color::Rgb(125, 174, 247),
            success: Color::Rgb(158, 206, 106),
            warning: Color::Rgb(224, 175, 104),
            error: Color::Rgb(247, 118, 142),
            info: Color::Rgb(125, 174, 247),

            load_low: Color::Rgb(158, 206, 106),
            load_medium: Color::Rgb(224, 175, 104),
            load_high: Color::Rgb(247, 118, 142),

            search_active: Color::Rgb(224, 175, 104),
            hint_key_fg: Color::Rgb(26, 27, 38),
            hint_key_bg: Color::Rgb(68, 75, 106),
            popup_bg: Color::Rgb(22, 22, 30),
            secure_core: Color::Rgb(187, 154, 247),

            help_bg: Color::Rgb(22, 22, 30),
            help_border: Color::Rgb(56, 62, 90),
            help_accent: Color::Rgb(125, 174, 247),
            help_section: Color::Rgb(158, 206, 106),
            help_key_fg: Color::Rgb(224, 175, 104),
            help_key_bg: Color::Rgb(41, 46, 66),
            help_desc: Color::Rgb(192, 202, 245),
            help_divider: Color::Rgb(41, 46, 66),
            help_footer_bg: Color::Rgb(41, 46, 66),
            help_footer_fg: Color::Rgb(169, 177, 214),

            upload: Color::Rgb(187, 154, 247),
        }
    }
}

mod color_serde {
    use ratatui::style::Color;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match color {
            Color::Black => "black".to_string(),
            Color::Red => "red".to_string(),
            Color::Green => "green".to_string(),
            Color::Yellow => "yellow".to_string(),
            Color::Blue => "blue".to_string(),
            Color::Magenta => "magenta".to_string(),
            Color::Cyan => "cyan".to_string(),
            Color::Gray => "gray".to_string(),
            Color::DarkGray => "darkgray".to_string(),
            Color::LightRed => "lightred".to_string(),
            Color::LightGreen => "lightgreen".to_string(),
            Color::LightYellow => "lightyellow".to_string(),
            Color::LightBlue => "lightblue".to_string(),
            Color::LightMagenta => "lightmagenta".to_string(),
            Color::LightCyan => "lightcyan".to_string(),
            Color::White => "white".to_string(),
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            Color::Indexed(i) => format!("indexed:{}", i),
            _ => "white".to_string(),
        };
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_color(&s).map_err(serde::de::Error::custom)
    }

    fn parse_color(s: &str) -> Result<Color, String> {
        match s.to_lowercase().as_str() {
            "black" => Ok(Color::Black),
            "red" => Ok(Color::Red),
            "green" => Ok(Color::Green),
            "yellow" => Ok(Color::Yellow),
            "blue" => Ok(Color::Blue),
            "magenta" => Ok(Color::Magenta),
            "cyan" => Ok(Color::Cyan),
            "gray" | "grey" => Ok(Color::Gray),
            "darkgray" | "darkgrey" | "dark_gray" | "dark_grey" => Ok(Color::DarkGray),
            "lightred" | "light_red" => Ok(Color::LightRed),
            "lightgreen" | "light_green" => Ok(Color::LightGreen),
            "lightyellow" | "light_yellow" => Ok(Color::LightYellow),
            "lightblue" | "light_blue" => Ok(Color::LightBlue),
            "lightmagenta" | "light_magenta" => Ok(Color::LightMagenta),
            "lightcyan" | "light_cyan" => Ok(Color::LightCyan),
            "white" => Ok(Color::White),
            s if s.starts_with('#') && s.len() == 7 => {
                let r = u8::from_str_radix(&s[1..3], 16)
                    .map_err(|e| format!("invalid red component: {}", e))?;
                let g = u8::from_str_radix(&s[3..5], 16)
                    .map_err(|e| format!("invalid green component: {}", e))?;
                let b = u8::from_str_radix(&s[5..7], 16)
                    .map_err(|e| format!("invalid blue component: {}", e))?;
                Ok(Color::Rgb(r, g, b))
            }
            s if s.starts_with("indexed:") => {
                let idx: u8 = s[8..]
                    .parse()
                    .map_err(|e| format!("invalid color index: {}", e))?;
                Ok(Color::Indexed(idx))
            }
            _ => Err(format!("unknown color: {}", s)),
        }
    }
}
