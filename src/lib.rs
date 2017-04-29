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
    if validate_phone_ISO3166(&number, iso3166) {
        return Ok(Cow::Owned("+".to_owned() + &number));
    }
    return Err(NotValidPhoneNumberError);
}

#[test]
fn test_parse(){
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
