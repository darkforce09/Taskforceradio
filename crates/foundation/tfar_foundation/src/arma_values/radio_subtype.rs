//! `RadioSubtype`: the kind of radio a push-to-talk command names.

#[cfg(test)]
#[path = "tests/radio_subtype.rs"]
mod tests;

/// The kind of radio a push-to-talk command names, read by its exact wire name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RadioSubtype {
    /// A short-range digital radio: `digital`.
    Digital,
    /// A long-range digital radio: `digital_lr`.
    DigitalLongRange,
    /// A diver radio: `dd`.
    DiverDigital,
    /// An airborne radio: `airborne`.
    Airborne,
    /// A phone: `phone`.
    Phone,
    /// Any other name, kept as sent; it plays no radio sound.
    Unknown(String),
}

impl RadioSubtype {
    /// The subtype whose wire name is exactly `name`; any other text is
    /// [`RadioSubtype::Unknown`]. The match is case-sensitive, as the game sends lowercase names.
    pub fn parse(name: &str) -> Self {
        match name {
            "digital" => Self::Digital,
            "digital_lr" => Self::DigitalLongRange,
            "dd" => Self::DiverDigital,
            "airborne" => Self::Airborne,
            "phone" => Self::Phone,
            other => Self::Unknown(other.to_owned()),
        }
    }

    /// The wire name: `digital`, `digital_lr`, `dd`, `airborne`, `phone`, or the text an unknown
    /// subtype was read from.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Digital => "digital",
            Self::DigitalLongRange => "digital_lr",
            Self::DiverDigital => "dd",
            Self::Airborne => "airborne",
            Self::Phone => "phone",
            Self::Unknown(name) => name,
        }
    }

    /// The folder under the plugin's sound folder that holds this subtype's radio sounds: `sw`,
    /// `lr`, `dd` or `ab`; `None` for a phone and an unknown subtype, which have none.
    pub const fn sound_folder(&self) -> Option<&'static str> {
        match self {
            Self::Digital => Some("sw"),
            Self::DigitalLongRange => Some("lr"),
            Self::DiverDigital => Some("dd"),
            Self::Airborne => Some("ab"),
            Self::Phone | Self::Unknown(_) => None,
        }
    }
}
