//! Region grouping for server display.
//!
//! Derives region from server name patterns (e.g., `US-TX#123`, `CA-ON#5`)
//! with city-to-region fallback for free servers.

/// Countries that have region grouping support.
const SUPPORTED_COUNTRIES: &[&str] = &["US", "CA", "DE", "GB", "AU"];

/// Check if a country has region grouping support.
pub fn is_supported_country(country_code: &str) -> bool {
    SUPPORTED_COUNTRIES.contains(&country_code)
}

/// Parse a region code from a server name like `US-TX#123` or `CA-ON#5`.
/// Returns `None` for unsupported countries or FREE patterns.
pub fn parse_region_from_name<'a>(country_code: &str, name: &'a str) -> Option<&'a str> {
    let prefix = format!("{}-", country_code);
    let rest = name.strip_prefix(&prefix)?;
    let code = rest.split('#').next()?;
    if code.len() != 2 || code == "FR" {
        return None;
    }
    // Validate it's actually a known region code
    if get_region_name(country_code, code).is_some() {
        Some(&rest[..2])
    } else {
        None
    }
}

/// Look up the full region name for a country + two-letter region code.
pub fn get_region_name(country_code: &str, code: &str) -> Option<&'static str> {
    match country_code {
        "US" => get_us_state_name(code),
        "CA" => get_ca_province_name(code),
        "DE" => get_de_state_name(code),
        "GB" => get_gb_region_name(code),
        "AU" => get_au_state_name(code),
        _ => None,
    }
}

/// Fallback: map a city name to its region code for the given country.
pub fn city_to_region(country_code: &str, city: &str) -> Option<&'static str> {
    match country_code {
        "US" => city_to_us_state(city),
        "CA" => city_to_ca_province(city),
        "DE" => city_to_de_state(city),
        "GB" => city_to_gb_region(city),
        "AU" => city_to_au_state(city),
        _ => None,
    }
}

/// Resolve region info for a server.
/// Returns `(region_code, region_name)` for supported countries.
pub fn resolve_region(
    country_code: &str,
    server_name: &str,
    city: &str,
) -> Option<(String, String)> {
    if !is_supported_country(country_code) {
        return None;
    }

    // Try parsing from server name first (e.g., US-TX#123, CA-ON#5)
    if let Some(code) = parse_region_from_name(country_code, server_name) {
        if let Some(name) = get_region_name(country_code, code) {
            return Some((code.to_string(), name.to_string()));
        }
    }

    // Fallback: city-to-region mapping (for FREE servers etc.)
    if let Some(code) = city_to_region(country_code, city) {
        if let Some(name) = get_region_name(country_code, code) {
            return Some((code.to_string(), name.to_string()));
        }
    }

    None
}

// --- US States ---

fn get_us_state_name(code: &str) -> Option<&'static str> {
    match code {
        "AL" => Some("Alabama"),
        "AK" => Some("Alaska"),
        "AZ" => Some("Arizona"),
        "AR" => Some("Arkansas"),
        "CA" => Some("California"),
        "CO" => Some("Colorado"),
        "CT" => Some("Connecticut"),
        "DE" => Some("Delaware"),
        "FL" => Some("Florida"),
        "GA" => Some("Georgia"),
        "HI" => Some("Hawaii"),
        "ID" => Some("Idaho"),
        "IL" => Some("Illinois"),
        "IN" => Some("Indiana"),
        "IA" => Some("Iowa"),
        "KS" => Some("Kansas"),
        "KY" => Some("Kentucky"),
        "LA" => Some("Louisiana"),
        "ME" => Some("Maine"),
        "MD" => Some("Maryland"),
        "MA" => Some("Massachusetts"),
        "MI" => Some("Michigan"),
        "MN" => Some("Minnesota"),
        "MS" => Some("Mississippi"),
        "MO" => Some("Missouri"),
        "MT" => Some("Montana"),
        "NE" => Some("Nebraska"),
        "NV" => Some("Nevada"),
        "NH" => Some("New Hampshire"),
        "NJ" => Some("New Jersey"),
        "NM" => Some("New Mexico"),
        "NY" => Some("New York"),
        "NC" => Some("North Carolina"),
        "ND" => Some("North Dakota"),
        "OH" => Some("Ohio"),
        "OK" => Some("Oklahoma"),
        "OR" => Some("Oregon"),
        "PA" => Some("Pennsylvania"),
        "RI" => Some("Rhode Island"),
        "SC" => Some("South Carolina"),
        "SD" => Some("South Dakota"),
        "TN" => Some("Tennessee"),
        "TX" => Some("Texas"),
        "UT" => Some("Utah"),
        "VT" => Some("Vermont"),
        "VA" => Some("Virginia"),
        "WA" => Some("Washington"),
        "WV" => Some("West Virginia"),
        "WI" => Some("Wisconsin"),
        "WY" => Some("Wyoming"),
        "DC" => Some("District of Columbia"),
        "PR" => Some("Puerto Rico"),
        _ => None,
    }
}

fn city_to_us_state(city: &str) -> Option<&'static str> {
    match city {
        "New York" | "New York City" | "Brooklyn" | "Buffalo" => Some("NY"),
        "Los Angeles" | "San Francisco" | "San Jose" | "San Diego" | "Sacramento" | "Palo Alto"
        | "Fremont" | "Oakland" => Some("CA"),
        "Chicago" => Some("IL"),
        "Dallas" | "Houston" | "San Antonio" | "Austin" | "Fort Worth" => Some("TX"),
        "Miami" | "Tampa" | "Orlando" | "Jacksonville" | "Fort Lauderdale" => Some("FL"),
        "Seattle" | "Tacoma" => Some("WA"),
        "Atlanta" => Some("GA"),
        "Denver" | "Boulder" => Some("CO"),
        "Phoenix" | "Scottsdale" | "Tucson" => Some("AZ"),
        "Las Vegas" | "Reno" => Some("NV"),
        "Portland" => Some("OR"),
        "Detroit" => Some("MI"),
        "Minneapolis" | "Saint Paul" => Some("MN"),
        "Salt Lake City" => Some("UT"),
        "Charlotte" | "Raleigh" => Some("NC"),
        "Boston" | "Cambridge" => Some("MA"),
        "Nashville" | "Memphis" => Some("TN"),
        "Kansas City" | "St. Louis" | "Saint Louis" => Some("MO"),
        "Columbus" | "Cleveland" | "Cincinnati" => Some("OH"),
        "Indianapolis" => Some("IN"),
        "Milwaukee" | "Madison" => Some("WI"),
        "Pittsburgh" | "Philadelphia" => Some("PA"),
        "Baltimore" => Some("MD"),
        "Washington" | "Washington, D.C." | "Washington DC" => Some("DC"),
        "Honolulu" => Some("HI"),
        "New Orleans" | "Baton Rouge" => Some("LA"),
        "Oklahoma City" | "Tulsa" => Some("OK"),
        "Albuquerque" | "Santa Fe" => Some("NM"),
        "Ashburn" | "Manassas" | "Norfolk" | "Richmond" | "Virginia Beach" => Some("VA"),
        "Newark" | "Jersey City" => Some("NJ"),
        "Hartford" | "New Haven" | "Stamford" => Some("CT"),
        "Providence" => Some("RI"),
        "Wilmington" => Some("DE"),
        "Charleston" | "Columbia" => Some("SC"),
        "Louisville" => Some("KY"),
        "Anchorage" => Some("AK"),
        "Boise" => Some("ID"),
        _ => None,
    }
}

// --- Canada Provinces/Territories ---

fn get_ca_province_name(code: &str) -> Option<&'static str> {
    match code {
        "ON" => Some("Ontario"),
        "BC" => Some("British Columbia"),
        "QC" => Some("Quebec"),
        "AB" => Some("Alberta"),
        "MB" => Some("Manitoba"),
        "SK" => Some("Saskatchewan"),
        "NS" => Some("Nova Scotia"),
        "NB" => Some("New Brunswick"),
        "NL" => Some("Newfoundland"),
        "PE" => Some("Prince Edward Island"),
        "NT" => Some("Northwest Territories"),
        "YT" => Some("Yukon"),
        "NU" => Some("Nunavut"),
        _ => None,
    }
}

fn city_to_ca_province(city: &str) -> Option<&'static str> {
    match city {
        "Toronto" | "Ottawa" | "Mississauga" | "Hamilton" => Some("ON"),
        "Vancouver" | "Victoria" | "Surrey" => Some("BC"),
        "Montreal" | "Quebec City" => Some("QC"),
        "Calgary" | "Edmonton" => Some("AB"),
        "Winnipeg" => Some("MB"),
        "Saskatoon" | "Regina" => Some("SK"),
        "Halifax" => Some("NS"),
        _ => None,
    }
}

// --- Germany (Bundesländer) ---

fn get_de_state_name(code: &str) -> Option<&'static str> {
    match code {
        "BY" => Some("Bavaria"),
        "BE" => Some("Berlin"),
        "HH" => Some("Hamburg"),
        "HE" => Some("Hesse"),
        "NW" => Some("North Rhine-Westphalia"),
        "BW" => Some("Baden-Württemberg"),
        "NI" => Some("Lower Saxony"),
        "SN" => Some("Saxony"),
        "RP" => Some("Rhineland-Palatinate"),
        "SH" => Some("Schleswig-Holstein"),
        "TH" => Some("Thuringia"),
        "BB" => Some("Brandenburg"),
        "MV" => Some("Mecklenburg-Vorpommern"),
        "SL" => Some("Saarland"),
        "ST" => Some("Saxony-Anhalt"),
        "HB" => Some("Bremen"),
        _ => None,
    }
}

fn city_to_de_state(city: &str) -> Option<&'static str> {
    match city {
        "Berlin" => Some("BE"),
        "Munich" => Some("BY"),
        "Frankfurt" => Some("HE"),
        "Hamburg" => Some("HH"),
        "Cologne" | "Düsseldorf" | "Dusseldorf" | "Dortmund" | "Essen" | "Bonn" => Some("NW"),
        "Stuttgart" => Some("BW"),
        "Hanover" | "Hannover" => Some("NI"),
        "Dresden" | "Leipzig" => Some("SN"),
        "Nuremberg" | "Nürnberg" => Some("BY"),
        "Bremen" => Some("HB"),
        _ => None,
    }
}

// --- United Kingdom ---

fn get_gb_region_name(code: &str) -> Option<&'static str> {
    match code {
        "EN" => Some("England"),
        "SC" => Some("Scotland"),
        "WA" => Some("Wales"),
        "NI" => Some("Northern Ireland"),
        // ProtonVPN may use city-based codes
        "LO" => Some("London"),
        "MA" => Some("Manchester"),
        "ED" => Some("Edinburgh"),
        _ => None,
    }
}

fn city_to_gb_region(city: &str) -> Option<&'static str> {
    match city {
        "London" | "Birmingham" | "Leeds" | "Bristol" | "Liverpool" | "Manchester"
        | "Sheffield" | "Nottingham" => Some("EN"),
        "Edinburgh" | "Glasgow" => Some("SC"),
        "Cardiff" | "Swansea" => Some("WA"),
        "Belfast" => Some("NI"),
        _ => None,
    }
}

// --- Australia ---

fn get_au_state_name(code: &str) -> Option<&'static str> {
    match code {
        "NS" => Some("New South Wales"),
        "VI" => Some("Victoria"),
        "QL" => Some("Queensland"),
        "WA" => Some("Western Australia"),
        "SA" => Some("South Australia"),
        "TA" => Some("Tasmania"),
        "AC" => Some("Australian Capital Territory"),
        "NT" => Some("Northern Territory"),
        _ => None,
    }
}

fn city_to_au_state(city: &str) -> Option<&'static str> {
    match city {
        "Sydney" | "Newcastle" | "Wollongong" => Some("NS"),
        "Melbourne" | "Geelong" => Some("VI"),
        "Brisbane" | "Gold Coast" => Some("QL"),
        "Perth" => Some("WA"),
        "Adelaide" => Some("SA"),
        "Hobart" => Some("TA"),
        "Canberra" => Some("AC"),
        "Darwin" => Some("NT"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // US tests
    #[test]
    fn test_parse_region_us() {
        assert_eq!(parse_region_from_name("US", "US-TX#123"), Some("TX"));
        assert_eq!(parse_region_from_name("US", "US-CA#1"), Some("CA"));
        assert_eq!(parse_region_from_name("US", "US-FREE#123"), None);
        assert_eq!(parse_region_from_name("US", "US-ZZ#1"), None);
    }

    #[test]
    fn test_resolve_region_us() {
        let r = resolve_region("US", "US-TX#5", "Dallas");
        assert_eq!(r, Some(("TX".to_string(), "Texas".to_string())));

        let r = resolve_region("US", "US-FREE#10", "Chicago");
        assert_eq!(r, Some(("IL".to_string(), "Illinois".to_string())));
    }

    #[test]
    fn test_us_city_fallback_new() {
        assert_eq!(city_to_us_state("Tucson"), Some("AZ"));
        assert_eq!(city_to_us_state("St. Louis"), Some("MO"));
        assert_eq!(city_to_us_state("Louisville"), Some("KY"));
        assert_eq!(city_to_us_state("Anchorage"), Some("AK"));
        assert_eq!(city_to_us_state("Boise"), Some("ID"));
        assert_eq!(city_to_us_state("Fort Worth"), Some("TX"));
        assert_eq!(city_to_us_state("Fort Lauderdale"), Some("FL"));
    }

    // Canada tests
    #[test]
    fn test_parse_region_canada() {
        assert_eq!(parse_region_from_name("CA", "CA-ON#5"), Some("ON"));
        assert_eq!(parse_region_from_name("CA", "CA-BC#1"), Some("BC"));
        assert_eq!(parse_region_from_name("CA", "CA-FREE#3"), None);
        assert_eq!(parse_region_from_name("CA", "CA-ZZ#1"), None);
    }

    #[test]
    fn test_resolve_region_canada() {
        let r = resolve_region("CA", "CA-ON#1", "Toronto");
        assert_eq!(r, Some(("ON".to_string(), "Ontario".to_string())));

        let r = resolve_region("CA", "CA-FREE#1", "Vancouver");
        assert_eq!(r, Some(("BC".to_string(), "British Columbia".to_string())));

        let r = resolve_region("CA", "CA-QC#3", "Montreal");
        assert_eq!(r, Some(("QC".to_string(), "Quebec".to_string())));
    }

    // Germany tests
    #[test]
    fn test_parse_region_germany() {
        assert_eq!(parse_region_from_name("DE", "DE-BE#1"), Some("BE"));
        assert_eq!(parse_region_from_name("DE", "DE-BY#3"), Some("BY"));
        assert_eq!(parse_region_from_name("DE", "DE-FREE#1"), None);
    }

    #[test]
    fn test_resolve_region_germany() {
        let r = resolve_region("DE", "DE-BE#1", "Berlin");
        assert_eq!(r, Some(("BE".to_string(), "Berlin".to_string())));

        let r = resolve_region("DE", "DE-FREE#1", "Munich");
        assert_eq!(r, Some(("BY".to_string(), "Bavaria".to_string())));

        let r = resolve_region("DE", "DE-HE#2", "Frankfurt");
        assert_eq!(r, Some(("HE".to_string(), "Hesse".to_string())));
    }

    // UK tests
    #[test]
    fn test_parse_region_uk() {
        assert_eq!(parse_region_from_name("GB", "GB-EN#1"), Some("EN"));
        assert_eq!(parse_region_from_name("GB", "GB-SC#2"), Some("SC"));
    }

    #[test]
    fn test_resolve_region_uk() {
        let r = resolve_region("GB", "GB-FREE#1", "London");
        assert_eq!(r, Some(("EN".to_string(), "England".to_string())));

        let r = resolve_region("GB", "GB-FREE#2", "Edinburgh");
        assert_eq!(r, Some(("SC".to_string(), "Scotland".to_string())));
    }

    // Australia tests
    #[test]
    fn test_parse_region_australia() {
        assert_eq!(parse_region_from_name("AU", "AU-NS#1"), Some("NS"));
        assert_eq!(parse_region_from_name("AU", "AU-VI#3"), Some("VI"));
    }

    #[test]
    fn test_resolve_region_australia() {
        let r = resolve_region("AU", "AU-NS#1", "Sydney");
        assert_eq!(r, Some(("NS".to_string(), "New South Wales".to_string())));

        let r = resolve_region("AU", "AU-FREE#1", "Melbourne");
        assert_eq!(r, Some(("VI".to_string(), "Victoria".to_string())));

        let r = resolve_region("AU", "AU-QL#2", "Brisbane");
        assert_eq!(r, Some(("QL".to_string(), "Queensland".to_string())));
    }

    // Unsupported country
    #[test]
    fn test_unsupported_country() {
        assert_eq!(resolve_region("FR", "FR-PA#1", "Paris"), None);
        assert_eq!(resolve_region("JP", "JP-TK#1", "Tokyo"), None);
    }

    // Cross-country: don't match wrong country prefix
    #[test]
    fn test_cross_country_no_match() {
        // US prefix shouldn't match for CA
        assert_eq!(parse_region_from_name("CA", "US-TX#1"), None);
        assert_eq!(parse_region_from_name("US", "CA-ON#1"), None);
    }
}
