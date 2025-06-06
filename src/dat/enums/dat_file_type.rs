use strum::Display;

#[derive(Debug, Display, PartialEq)]
pub enum DatFileType {
    Texture,
    Unknown,
}

#[derive(Debug, Display, PartialEq)]
pub enum DatFileSubtype {
    Icon,
    Unknown,
}
