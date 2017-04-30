#![feature(str_escape)]
extern crate regex;

mod iso3166;

use std::borrow::Cow;
use std::error;
use std::fmt;
use regex::Regex;
use iso3166::*;

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

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

pub fn parse<'a>(number: &str, country: &str) -> Result<Cow<'a, str>, NotValidPhoneNumberError> {
    let number = number.trim();
    let country = country.trim();
    let has_plus_sign = number.starts_with("+");

    // remove any non-digit character, included the +
    let mut number: String = Regex::new(r"\D").unwrap().replace_all(number, "").escape_default();
    let mut iso3166 = try!(get_iso3166_by_country(&country).ok_or(NotValidPhoneNumberError));

    if !vec![ISO3166_GA.alpha3, ISO3166_CI.alpha3, ISO3166_CG.alpha3].contains(&iso3166.alpha3) {
        number = Regex::new(r"^0+").unwrap().replace_all(&number, "").escape_default();
    }

    if iso3166.alpha3 == ISO3166_RU.alpha3 && number.len() == 11 &&
       Regex::new(r"^89").unwrap().is_match(&number) {
        number = Regex::new(r"^8+").unwrap().replace_all(&number, "").escape_default();
    }

    if has_plus_sign {
        iso3166 = try!(get_iso3166_by_number(&number).ok_or(NotValidPhoneNumberError));
    } else {
        if iso3166.phone_number_lengths.contains(&number.len()) {
            number = iso3166.country_code.to_owned() + &number;
        }
    }
    if validate_phone_iso3166(&number, iso3166) {
        return Ok(Cow::Owned("+".to_owned() + &number));
    }
    return Err(NotValidPhoneNumberError);
}


pub fn get_iso3166_by_country(country: &str) -> Option<&'static ISO3166> {
    let country = country.to_uppercase();
    let l = country.len();
    match l {
        2 => {
            return ISO3166S.into_iter().filter(|iso3166| iso3166.alpha2 == country).next();
        }
        3 => {
            return ISO3166S.into_iter().filter(|iso3166| iso3166.alpha3 == country).next();
        }
        l if 4 < l => {
            return ISO3166S.into_iter()
                .filter(|iso3166| iso3166.country_name.to_uppercase() == country)
                .next();
        }
        _ => return None,
    }
}

pub fn get_iso3166_by_number(number: &str) -> Option<&'static ISO3166> {
    for phone in ISO3166S {
        let r = Regex::new(("^".to_owned() + phone.country_code).as_str()).unwrap();
        for l in phone.phone_number_lengths {
            if r.is_match(number) && number.len() == phone.country_code.len() + l {
                for mbw in phone.mobile_begin_with {
                    if Regex::new(("^".to_owned() + phone.country_code + mbw).as_str())
                        .unwrap()
                        .is_match(number) {
                        return Some(phone);
                    }
                }
            }
        }
    }
    return None;
}

pub fn validate_phone_iso3166(number: &str, iso3166: &ISO3166) -> bool {
    if iso3166.phone_number_lengths.len() == 0 {
        return false;
    }
    let number = Regex::new(("^".to_owned() + iso3166.country_code).as_str())
        .unwrap()
        .replace_all(number, "");
    for l in iso3166.phone_number_lengths {
        if l == &number.len() {
            for mbw in iso3166.mobile_begin_with {
                if Regex::new(("^".to_owned() + mbw).as_str()).unwrap().is_match(&number) {
                    return true;
                }
            }
        }
    }
    return false;
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
            Err(err) => panic!(err),
        }
        let iso3166 = parse("892 3456 7890", "ru");
        match iso3166 {
            Ok(ru) => assert_eq!("+79234567890", ru),
            Err(err) => panic!(err),
        }
    }
}
