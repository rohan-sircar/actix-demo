use lazy_static::lazy_static;
use regex::Regex;
lazy_static! {
    pub static ref USERNAME_REG: Regex =
        Regex::new(r"^([a-z\d.]+-)*[a-z\d.]+{5,35}$").unwrap();
    pub static ref PETNAME_REG: Regex =
        Regex::new(r"^[A-Z\sa-z]+{5,35}$").unwrap();
}
