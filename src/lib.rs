extern crate regex;

mod iso3166;

use std::error;
use std::fmt;
use std::sync::LazyLock;

use regex::Regex;

use crate::iso3166::*;

static LAZY_REGEX_D: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\D").unwrap());
static LAZY_REGEX_0: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^0+").unwrap());
static LAZY_REGEX_8: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^8+").unwrap());
static LAZY_REGEX_89: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^89+").unwrap());

#[derive(Debug)]
pub struct NotValidPhoneNumberError;

impl fmt::Display for NotValidPhoneNumberError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "It is not a valid phone number")
    }
}

impl error::Error for NotValidPhoneNumberError {
    fn description(&self) -> &str {
        "It is not a valid phone number"
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

pub fn parse(number: &str, country: &str) -> Result<String, NotValidPhoneNumberError> {
    let number = number.trim();
    let country = country.trim();

    // remove any non-digit character, included the +
    let mut number: String = LAZY_REGEX_D.replace_all(number, "").to_string();
    let mut iso3166 = get_iso3166_by_country(country).ok_or(NotValidPhoneNumberError)?;

    if ![ISO3166_GA.alpha3, ISO3166_CI.alpha3, ISO3166_CG.alpha3].contains(&iso3166.alpha3) {
        number = LAZY_REGEX_0.replace_all(&number, "").to_string();
    }

    if iso3166.alpha3 == ISO3166_RU.alpha3 && number.len() == 11 && LAZY_REGEX_89.is_match(&number)
    {
        number = LAZY_REGEX_8.replace_all(&number, "").to_string()
    }

    if number.starts_with("+") {
        iso3166 = get_iso3166_by_number(&number).ok_or(NotValidPhoneNumberError)?;
    } else if iso3166.phone_number_lengths.contains(&number.len()) {
        number = iso3166.country_code.to_owned() + &number;
    }
    if validate_phone_iso3166(&number, iso3166) {
        return Ok("+".to_owned() + number.as_str());
    }
    Err(NotValidPhoneNumberError)
}

pub fn get_iso3166_by_country(country: &str) -> Option<&'static ISO3166> {
    match country.len() {
        2 => ISO3166S
            .iter()
            .find(|iso3166| iso3166.alpha2 == country.to_uppercase()),
        3 => ISO3166S
            .iter()
            .find(|iso3166| iso3166.alpha3 == country.to_uppercase()),
        l if 4 < l => ISO3166S
            .iter()
            .find(|iso3166| iso3166.country_name.to_uppercase() == country.to_uppercase()),
        _ => None,
    }
}

pub fn get_iso3166_by_number(number: &str) -> Option<&'static ISO3166> {
    for phone in ISO3166S {
        for l in phone.phone_number_lengths {
            if number.starts_with(phone.country_code)
                && number.len() == phone.country_code.len() + l
            {
                for &mbw in phone.mobile_begin_with {
                    if Regex::new(("^".to_owned() + phone.country_code + mbw).as_str())
                        .unwrap()
                        .is_match(number)
                    {
                        return Some(phone);
                    }
                }
            }
        }
    }
    None
}

pub fn validate_phone_iso3166(number: &str, iso3166: &ISO3166) -> bool {
    if iso3166.phone_number_lengths.is_empty() {
        return false;
    }
    let number = Regex::new(("^".to_owned() + iso3166.country_code).as_str())
        .unwrap()
        .replace_all(number, "");
    for l in iso3166.phone_number_lengths {
        if l == &number.len() {
            for mbw in iso3166.mobile_begin_with {
                if number.starts_with(mbw) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_iso3166_by_country() {
        let iso3166 = get_iso3166_by_country("JP");
        match iso3166 {
            Some(jp) => assert!(jp == &ISO3166_JP),
            None => panic!("None"),
        }
    }

    #[test]
    fn test_parse() {
        let iso3166 = parse("090 0000 0000", "jp");
        match iso3166 {
            Ok(jp) => assert_eq!("+819000000000", jp),
            Err(err) => std::panic::panic_any(err),
        }
        let iso3166 = parse("892 3456 7890", "ru");
        match iso3166 {
            Ok(ru) => assert_eq!("+79234567890", ru),
            Err(err) => std::panic::panic_any(err),
        }
    }
}
