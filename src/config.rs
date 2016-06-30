//! Configuration definitions and file loading
//!
//! Alacritty reads from a config file at startup to determine various runtime
//! parameters including font family and style, font size, etc. In the future,
//! the config file will also hold user and platform specific keybindings.
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde_yaml;

/// Top-level config type
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    /// Pixels per inch
    #[serde(default)]
    dpi: Dpi,

    /// Font configuration
    #[serde(default)]
    font: Font,

    /// Should show render timer
    #[serde(default)]
    render_timer: bool,
}

/// Errors occurring during config loading
#[derive(Debug)]
pub enum Error {
    /// Config file not found
    NotFound,

    /// Couldn't read $HOME environment variable
    ReadingEnvHome(env::VarError),

    /// io error reading file
    Io(io::Error),

    /// Not valid yaml or missing parameters
    Yaml(serde_yaml::Error),
}

impl ::std::error::Error for Error {
    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            Error::NotFound => None,
            Error::ReadingEnvHome(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::Yaml(ref err) => Some(err),
        }
    }

    fn description(&self) -> &str {
        match *self {
            Error::NotFound => "could not locate config file",
            Error::ReadingEnvHome(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
            Error::Yaml(ref err) => err.description(),
        }
    }
}

impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Error::NotFound => write!(f, "{}", ::std::error::Error::description(self)),
            Error::ReadingEnvHome(ref err) => {
                write!(f, "could not read $HOME environment variable: {}", err)
            },
            Error::Io(ref err) => write!(f, "error reading config file: {}", err),
            Error::Yaml(ref err) => write!(f, "problem with config: {}", err),
        }
    }
}

impl From<env::VarError> for Error {
    fn from(val: env::VarError) -> Error {
        Error::ReadingEnvHome(val)
    }
}

impl From<io::Error> for Error {
    fn from(val: io::Error) -> Error {
        if val.kind() == io::ErrorKind::NotFound {
            Error::NotFound
        } else {
            Error::Io(val)
        }
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(val: serde_yaml::Error) -> Error {
        Error::Yaml(val)
    }
}

/// Result from config loading
pub type Result<T> = ::std::result::Result<T, Error>;

impl Config {
    /// Attempt to load the config file
    ///
    /// The config file is loaded from the first file it finds in this list of paths
    ///
    /// 1. `$HOME/.config/alacritty.yml`
    /// 2. `$HOME/.alacritty.yml`
    pub fn load() -> Result<Config> {
        let home = env::var("HOME")?;

        // First path
        let mut path = PathBuf::from(&home);
        path.push(".config");
        path.push("alacritty.yml");

        // Fallback path
        let mut alt_path = PathBuf::from(&home);
        alt_path.push(".alacritty.yml");

        match Config::load_from(&path) {
            Ok(c) => Ok(c),
            Err(e) => {
                match e {
                    Error::NotFound => Config::load_from(&alt_path),
                    _ => Err(e),
                }
            }
        }
    }

    /// Get font config
    #[inline]
    pub fn font(&self) -> &Font {
        &self.font
    }

    /// Get dpi config
    #[inline]
    pub fn dpi(&self) -> &Dpi {
        &self.dpi
    }

    /// Should show render timer
    #[inline]
    pub fn render_timer(&self) -> bool {
        self.render_timer
    }

    fn load_from<P: AsRef<Path>>(path: P) -> Result<Config> {
        let raw = Config::read_file(path)?;
        Ok(serde_yaml::from_str(&raw[..])?)
    }

    fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
        let mut f = fs::File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;

        Ok(contents)
    }
}

/// Pixels per inch
///
/// This is only used on FreeType systems
#[derive(Debug, Deserialize)]
pub struct Dpi {
    /// Horizontal dpi
    x: f32,

    /// Vertical dpi
    y: f32,
}

impl Default for Dpi {
    fn default() -> Dpi {
        Dpi { x: 96.0, y: 96.0 }
    }
}

impl Dpi {
    /// Get horizontal dpi
    #[inline]
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Get vertical dpi
    #[inline]
    pub fn y(&self) -> f32 {
        self.y
    }
}

/// Modifications to font spacing
///
/// The way Alacritty calculates vertical and horizontal cell sizes may not be
/// ideal for all fonts. This gives the user a way to tweak those values.
#[derive(Debug, Deserialize)]
pub struct FontOffset {
    /// Extra horizontal spacing between letters
    x: f32,
    /// Extra vertical spacing between lines
    y: f32,
}

impl FontOffset {
    /// Get letter spacing
    #[inline]
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Get line spacing
    #[inline]
    pub fn y(&self) -> f32 {
        self.y
    }
}

/// Font config
///
/// Defaults are provided at the level of this struct per platform, but not per
/// field in this struct. It might be nice in the future to have defaults for
/// each value independently. Alternatively, maybe erroring when the user
/// doesn't provide complete config is Ok.
#[derive(Debug, Deserialize)]
pub struct Font {
    /// Font family
    family: String,

    /// Font style
    style: String,

    /// Font size in points
    size: f32,

    /// Extra spacing per character
    offset: FontOffset,
}

impl Font {
    /// Get the font family
    #[inline]
    pub fn family(&self) -> &str {
        &self.family[..]
    }

    /// Get the font style
    #[inline]
    pub fn style(&self) -> &str {
        &self.style[..]
    }

    /// Get the font size in points
    #[inline]
    pub fn size(&self) -> f32 {
        self.size
    }

    /// Get offsets to font metrics
    #[inline]
    pub fn offset(&self) -> &FontOffset {
        &self.offset
    }
}

#[cfg(target_os = "macos")]
impl Default for Font {
    fn default() -> Font {
        Font {
            family: String::from("Menlo"),
            style: String::from("Regular"),
            size: 11.0,
            offset: FontOffset {
                x: 0.0,
                y: 0.0
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Default for Font {
    fn default() -> Font {
        Font {
            family: String::from("DejaVu Sans Mono"),
            style: String::from("Book"),
            size: 11.0,
            offset: FontOffset {
                // TODO should improve freetype metrics... shouldn't need such
                // drastic offsets for the default!
                x: 2.0,
                y: -7.0
            }
        }
    }
}