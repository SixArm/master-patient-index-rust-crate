//! Patient matching algorithms
//!
//! This module implements various matching algorithms for comparing patient records:
//! - Name matching (fuzzy and phonetic)
//! - Date of birth matching
//! - Gender matching
//! - Address matching
//! - Identifier matching

use strsim::{jaro_winkler, levenshtein, normalized_levenshtein};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use chrono::{NaiveDate, Datelike};

use crate::models::{Patient, HumanName, Address, Identifier};

/// Name matching algorithms
pub mod name_matching {
    use super::*;

    /// Calculate similarity between two names using multiple algorithms
    pub fn match_names(name1: &HumanName, name2: &HumanName) -> f64 {
        // Weight factors for different components
        const FAMILY_WEIGHT: f64 = 0.5;
        const GIVEN_WEIGHT: f64 = 0.4;
        const PREFIX_SUFFIX_WEIGHT: f64 = 0.1;

        let family_score = match_family_names(&name1.family, &name2.family);
        let given_score = match_given_names(&name1.given, &name2.given);
        let prefix_suffix_score = match_prefix_suffix(
            &name1.prefix,
            &name2.prefix,
            &name1.suffix,
            &name2.suffix,
        );

        (family_score * FAMILY_WEIGHT)
            + (given_score * GIVEN_WEIGHT)
            + (prefix_suffix_score * PREFIX_SUFFIX_WEIGHT)
    }

    /// Match family names using fuzzy string matching
    pub fn match_family_names(family1: &str, family2: &str) -> f64 {
        if family1.is_empty() || family2.is_empty() {
            return 0.0;
        }

        // Normalize: lowercase and trim
        let f1 = family1.trim().to_lowercase();
        let f2 = family2.trim().to_lowercase();

        // Exact match
        if f1 == f2 {
            return 1.0;
        }

        // Use Jaro-Winkler (good for name matching)
        let jw_score = jaro_winkler(&f1, &f2);

        // Use normalized Levenshtein distance
        let lev_score = normalized_levenshtein(&f1, &f2);

        // Take the maximum score
        f64::max(jw_score, lev_score)
    }

    /// Match given names (array of names)
    pub fn match_given_names(given1: &[String], given2: &[String]) -> f64 {
        if given1.is_empty() || given2.is_empty() {
            return 0.0;
        }

        // Compare first names primarily
        let first1 = given1.first().unwrap().trim().to_lowercase();
        let first2 = given2.first().unwrap().trim().to_lowercase();

        if first1 == first2 {
            return 1.0;
        }

        // Check for common nicknames/variants
        if are_name_variants(&first1, &first2) {
            return 0.95;
        }

        // Fuzzy match
        let jw_score = jaro_winkler(&first1, &first2);
        let lev_score = normalized_levenshtein(&first1, &first2);

        f64::max(jw_score, lev_score)
    }

    /// Check if two names are known variants/nicknames
    fn are_name_variants(name1: &str, name2: &str) -> bool {
        // Common name variants (simplified list)
        let variants = [
            vec!["william", "bill", "billy", "will"],
            vec!["robert", "bob", "bobby", "rob"],
            vec!["richard", "dick", "rick", "ricky"],
            vec!["james", "jim", "jimmy", "jamie"],
            vec!["john", "jack", "johnny"],
            vec!["michael", "mike", "mickey"],
            vec!["elizabeth", "liz", "beth", "betty", "betsy"],
            vec!["margaret", "maggie", "meg", "peggy"],
            vec!["catherine", "cathy", "kate", "katie"],
            vec!["jennifer", "jen", "jenny"],
            vec!["christopher", "chris"],
            vec!["anthony", "tony"],
            vec!["thomas", "tom", "tommy"],
            vec!["joseph", "joe", "joey"],
            vec!["charles", "charlie", "chuck"],
        ];

        for variant_group in &variants {
            if variant_group.contains(&name1) && variant_group.contains(&name2) {
                return true;
            }
        }

        false
    }

    /// Match prefix and suffix arrays
    fn match_prefix_suffix(
        prefix1: &[String],
        prefix2: &[String],
        suffix1: &[String],
        suffix2: &[String],
    ) -> f64 {
        let prefix_match = if prefix1.is_empty() && prefix2.is_empty() {
            1.0
        } else if prefix1.is_empty() || prefix2.is_empty() {
            0.5
        } else {
            // Check if any prefix matches
            let mut max_score = 0.0;
            for p1 in prefix1 {
                for p2 in prefix2 {
                    let score = jaro_winkler(
                        &p1.to_lowercase(),
                        &p2.to_lowercase(),
                    );
                    max_score = f64::max(max_score, score);
                }
            }
            max_score
        };

        let suffix_match = if suffix1.is_empty() && suffix2.is_empty() {
            1.0
        } else if suffix1.is_empty() || suffix2.is_empty() {
            0.5
        } else {
            // Check if any suffix matches
            let mut max_score = 0.0;
            for s1 in suffix1 {
                for s2 in suffix2 {
                    let score = jaro_winkler(
                        &s1.to_lowercase(),
                        &s2.to_lowercase(),
                    );
                    max_score = f64::max(max_score, score);
                }
            }
            max_score
        };

        (prefix_match + suffix_match) / 2.0
    }
}

/// Date of birth matching
pub mod dob_matching {
    use super::*;

    /// Match dates of birth with tolerance for data entry errors
    pub fn match_birth_dates(
        dob1: Option<NaiveDate>,
        dob2: Option<NaiveDate>,
    ) -> f64 {
        match (dob1, dob2) {
            (None, None) => 0.5, // Both missing - neutral
            (None, Some(_)) | (Some(_), None) => 0.0, // One missing - no match
            (Some(d1), Some(d2)) => {
                if d1 == d2 {
                    return 1.0; // Exact match
                }

                // Allow for common data entry errors
                let days_diff = (d1 - d2).num_days().abs();

                // Same month and year, day off by 1-2 (typo)
                if d1.year() == d2.year() && d1.month() == d2.month() {
                    if days_diff <= 2 {
                        return 0.95;
                    }
                }

                // Month/day transposition (e.g., 03/12 vs 12/03)
                if d1.year() == d2.year()
                    && d1.month() == d2.day()
                    && d1.day() == d2.month()
                {
                    return 0.90;
                }

                // Same year and month
                if d1.year() == d2.year() && d1.month() == d2.month() {
                    return 0.80;
                }

                // Same year, different month
                if d1.year() == d2.year() {
                    return 0.50;
                }

                // Year off by 1 (typo in year)
                if (d1.year() - d2.year()).abs() == 1
                    && d1.month() == d2.month()
                    && d1.day() == d2.day()
                {
                    return 0.85;
                }

                // No significant match
                0.0
            }
        }
    }
}

/// Gender matching
pub mod gender_matching {
    use crate::models::Gender;

    /// Match gender fields
    pub fn match_gender(gender1: Gender, gender2: Gender) -> f64 {
        if gender1 == gender2 {
            1.0
        } else if gender1 == Gender::Unknown || gender2 == Gender::Unknown {
            0.5 // Unknown is neutral
        } else {
            0.0 // Mismatch
        }
    }
}

/// Address matching
pub mod address_matching {
    use super::*;

    /// Match addresses using multiple components
    pub fn match_addresses(addresses1: &[Address], addresses2: &[Address]) -> f64 {
        if addresses1.is_empty() || addresses2.is_empty() {
            return 0.0;
        }

        // Compare primary addresses if available
        let addr1 = addresses1.first().unwrap();
        let addr2 = addresses2.first().unwrap();

        match_address(addr1, addr2)
    }

    /// Match individual addresses
    pub fn match_address(addr1: &Address, addr2: &Address) -> f64 {
        const POSTAL_CODE_WEIGHT: f64 = 0.3;
        const CITY_WEIGHT: f64 = 0.2;
        const STATE_WEIGHT: f64 = 0.2;
        const STREET_WEIGHT: f64 = 0.3;

        let postal_score = match_postal_codes(
            addr1.postal_code.as_deref(),
            addr2.postal_code.as_deref(),
        );

        let city_score = match_cities(
            addr1.city.as_deref(),
            addr2.city.as_deref(),
        );

        let state_score = match_states(
            addr1.state.as_deref(),
            addr2.state.as_deref(),
        );

        let street_score = match_street_addresses(
            addr1.line1.as_deref(),
            addr2.line1.as_deref(),
        );

        (postal_score * POSTAL_CODE_WEIGHT)
            + (city_score * CITY_WEIGHT)
            + (state_score * STATE_WEIGHT)
            + (street_score * STREET_WEIGHT)
    }

    /// Match postal codes
    pub(crate) fn match_postal_codes(zip1: Option<&str>, zip2: Option<&str>) -> f64 {
        match (zip1, zip2) {
            (None, None) => 0.0,
            (None, Some(_)) | (Some(_), None) => 0.0,
            (Some(z1), Some(z2)) => {
                let z1 = z1.trim().replace("-", "");
                let z2 = z2.trim().replace("-", "");

                if z1 == z2 {
                    return 1.0;
                }

                // Match first 5 digits (US ZIP)
                if z1.len() >= 5 && z2.len() >= 5 {
                    if &z1[0..5] == &z2[0..5] {
                        return 0.95;
                    }
                }

                // Match first 3 digits (same area)
                if z1.len() >= 3 && z2.len() >= 3 {
                    if &z1[0..3] == &z2[0..3] {
                        return 0.70;
                    }
                }

                0.0
            }
        }
    }

    /// Match cities
    fn match_cities(city1: Option<&str>, city2: Option<&str>) -> f64 {
        match (city1, city2) {
            (None, None) => 0.0,
            (None, Some(_)) | (Some(_), None) => 0.0,
            (Some(c1), Some(c2)) => {
                let c1 = c1.trim().to_lowercase();
                let c2 = c2.trim().to_lowercase();

                if c1 == c2 {
                    return 1.0;
                }

                // Fuzzy match for typos
                jaro_winkler(&c1, &c2)
            }
        }
    }

    /// Match states
    fn match_states(state1: Option<&str>, state2: Option<&str>) -> f64 {
        match (state1, state2) {
            (None, None) => 0.0,
            (None, Some(_)) | (Some(_), None) => 0.0,
            (Some(s1), Some(s2)) => {
                let s1 = s1.trim().to_uppercase();
                let s2 = s2.trim().to_uppercase();

                if s1 == s2 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    /// Match street addresses
    fn match_street_addresses(street1: Option<&str>, street2: Option<&str>) -> f64 {
        match (street1, street2) {
            (None, None) => 0.0,
            (None, Some(_)) | (Some(_), None) => 0.0,
            (Some(s1), Some(s2)) => {
                let s1 = normalize_street(s1);
                let s2 = normalize_street(s2);

                if s1 == s2 {
                    return 1.0;
                }

                // Fuzzy match
                jaro_winkler(&s1, &s2)
            }
        }
    }

    /// Normalize street address for comparison
    fn normalize_street(street: &str) -> String {
        street
            .trim()
            .to_lowercase()
            .replace("street", "st")
            .replace("avenue", "ave")
            .replace("road", "rd")
            .replace("drive", "dr")
            .replace("boulevard", "blvd")
            .replace("lane", "ln")
            .replace("court", "ct")
            .replace("circle", "cir")
            .replace(".", "")
            .replace(",", "")
    }
}

/// Identifier matching
pub mod identifier_matching {
    use super::*;

    /// Match patient identifiers
    pub fn match_identifiers(ids1: &[Identifier], ids2: &[Identifier]) -> f64 {
        if ids1.is_empty() || ids2.is_empty() {
            return 0.0;
        }

        let mut max_score = 0.0;

        for id1 in ids1 {
            for id2 in ids2 {
                let score = match_identifier(id1, id2);
                max_score = f64::max(max_score, score);
            }
        }

        max_score
    }

    /// Match individual identifiers
    pub fn match_identifier(id1: &Identifier, id2: &Identifier) -> f64 {
        // Must be same type and system
        if id1.identifier_type != id2.identifier_type {
            return 0.0;
        }

        if id1.system != id2.system {
            return 0.0;
        }

        // Compare values
        let v1 = id1.value.trim().to_lowercase();
        let v2 = id2.value.trim().to_lowercase();

        if v1 == v2 {
            1.0 // Exact match
        } else {
            // Allow minor differences (e.g., formatting)
            let v1_clean = v1.replace("-", "").replace(" ", "");
            let v2_clean = v2.replace("-", "").replace(" ", "");

            if v1_clean == v2_clean {
                0.98 // Formatting difference
            } else {
                0.0 // Different values
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_name_match() {
        let name1 = HumanName {
            use_type: None,
            family: "Smith".to_string(),
            given: vec!["John".to_string()],
            prefix: vec![],
            suffix: vec![],
        };

        let name2 = name1.clone();

        let score = name_matching::match_names(&name1, &name2);
        assert!(score > 0.99, "Exact match should score ~1.0, got {}", score);
    }

    #[test]
    fn test_fuzzy_name_match() {
        let name1 = HumanName {
            use_type: None,
            family: "Smith".to_string(),
            given: vec!["John".to_string()],
            prefix: vec![],
            suffix: vec![],
        };

        let name2 = HumanName {
            use_type: None,
            family: "Smyth".to_string(), // Spelling variant
            given: vec!["John".to_string()],
            prefix: vec![],
            suffix: vec![],
        };

        let score = name_matching::match_names(&name1, &name2);
        assert!(score > 0.85, "Similar names should score high, got {}", score);
    }

    #[test]
    fn test_name_variants() {
        let name1 = HumanName {
            use_type: None,
            family: "Smith".to_string(),
            given: vec!["William".to_string()],
            prefix: vec![],
            suffix: vec![],
        };

        let name2 = HumanName {
            use_type: None,
            family: "Smith".to_string(),
            given: vec!["Bill".to_string()],
            prefix: vec![],
            suffix: vec![],
        };

        let score = name_matching::match_names(&name1, &name2);
        assert!(score > 0.90, "Name variants should score high, got {}", score);
    }

    #[test]
    fn test_exact_dob_match() {
        let dob = NaiveDate::from_ymd_opt(1980, 1, 15);
        let score = dob_matching::match_birth_dates(dob, dob);
        assert_eq!(score, 1.0);
    }

    #[test]
    fn test_dob_typo() {
        let dob1 = NaiveDate::from_ymd_opt(1980, 1, 15);
        let dob2 = NaiveDate::from_ymd_opt(1980, 1, 16); // Day off by 1
        let score = dob_matching::match_birth_dates(dob1, dob2);
        assert!(score > 0.90, "Minor DOB typo should score high, got {}", score);
    }

    #[test]
    fn test_gender_match() {
        use crate::models::Gender;

        assert_eq!(gender_matching::match_gender(Gender::Male, Gender::Male), 1.0);
        assert_eq!(gender_matching::match_gender(Gender::Male, Gender::Female), 0.0);
        assert_eq!(gender_matching::match_gender(Gender::Male, Gender::Unknown), 0.5);
    }

    #[test]
    fn test_postal_code_match() {
        let score = address_matching::match_postal_codes(
            Some("12345"),
            Some("12345"),
        );
        assert_eq!(score, 1.0);

        let score = address_matching::match_postal_codes(
            Some("12345-6789"),
            Some("12345"),
        );
        assert!(score > 0.90);
    }
}
