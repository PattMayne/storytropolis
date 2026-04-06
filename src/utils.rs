/* 
 * ===============================
 * ===============================
 * =====                     =====
 * =====  UTILS AND HELPERS  =====
 * =====                     =====
 * ===============================
 * ===============================
 */



use regex::Regex;
use rand::Rng;
use rand_regex::Regex as RandRegex;


/* 
 * ===============================
 * ===============================
 * =====                     =====
 * =====  INPUT VALIDATIONS  =====
 * =====                     =====
 * ===============================
 * ===============================
 */

pub struct StringRange {
    pub min: usize,
    pub max: usize,
}

fn username_length_range() -> StringRange {
    StringRange{ min: 6, max: 20 }
}

fn password_length_range() -> StringRange {
    StringRange{ min: 6, max: 16 }
}

fn real_name_length_range() -> StringRange {
    StringRange { min: 2, max: 50 }
}

/**
 * Generate a purely alphanumeric "client secret" which is basically a password
 * for the client_sites.
 */
pub fn generate_client_secret() -> String {
    let mut rng: rand::prelude::ThreadRng = rand::rng();
    let reg: RandRegex = RandRegex::compile(
        r"(?i)[a-z0-9]{37}",
        100
    ).unwrap();
    
    rng.sample(reg)
}

pub fn string_length_valid(range_obj: StringRange, string: &String) -> bool {
    let string_length: usize = string.len();
    string_length >= range_obj.min && string_length <= range_obj.max
}

pub fn validate_username(username: &String) -> bool {
    let reg: Regex = Regex::new(r"^[A-Za-z0-9_-]+$").unwrap();
    reg.is_match(&username) &&
        string_length_valid(
            username_length_range(),
            &username)
}

pub fn validate_password(password: &String) -> bool {
    let reg: Regex = Regex::new(r"[A-Za-z0-9!@#$%^&*()_\-+=\[\]{}:;<>.,?~`|]+$").unwrap();
    reg.is_match(&password) &&
        string_length_valid(
            password_length_range(),
            &password)
}

pub fn has_no_whitespace(string: &String) -> bool {
    let reg: Regex = Regex::new(r"^\S+$").unwrap();
    reg.is_match(string)
}

pub fn validate_email(email: &String) -> bool {
    let reg: Regex = Regex::new(r"^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$").unwrap();
    reg.is_match(&email)
}

pub fn validate_real_name(name: &String) -> bool {
    string_length_valid(real_name_length_range(), name)
}

// TO DO: Move this into RESOURCES file
pub fn auth_client_id() -> String { String::from("auth_site") }

pub fn validate_url(url: &String) -> bool {
    let lenient_regex: Regex =
        Regex::new(r"^https?://[^\s/$.?#].[^\s]*$")
        .unwrap();
    // We might never use the strict regex
    let _strict_regex = 
        Regex::new(r"^https?://([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}(\/[^\s]*)?$")
        .unwrap();
    lenient_regex.is_match(url)
}



/* 
 * 
 * 
 * 
 * 
 * ==============================
 * ==============================
 * =====                    =====
 * =====  LANGUAGE SUPPORT  =====
 * =====                    =====
 * ==============================
 * ==============================
 * 
 * 
 * 
 * 
*/

/**
 * This enum goes into the req object
 * to deliver standard language suffixes
 * for the translations script.
 */
#[derive(Clone)]
pub enum SupportedLangs {
    French,
    English
}

impl SupportedLangs {
    pub fn suffix(&self) -> &'static str {
        match self {
            SupportedLangs::English => "en",
            SupportedLangs::French => "fr"
        }
    }

    // When checking the header (accept-lang) or DB for lang
    pub fn from(input: &str) -> SupportedLangs {
        if input.starts_with("en") {
            return SupportedLangs::English;
        } else if input.starts_with("fr") {
            return SupportedLangs::French;
        }

        // Default
        return SupportedLangs::English;
    }
}

