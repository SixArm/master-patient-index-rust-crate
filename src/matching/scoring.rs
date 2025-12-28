//! Match scoring calculations
//!
//! This module combines individual matching algorithm scores into
//! overall match scores using configurable weights.

use crate::models::Patient;
use crate::config::MatchingConfig;
use super::{MatchResult, MatchScoreBreakdown};
use super::algorithms::{
    name_matching, dob_matching, gender_matching,
    address_matching, identifier_matching,
};

/// Probabilistic scoring strategy
pub struct ProbabilisticScorer {
    /// Configuration for matching thresholds and weights
    config: MatchingConfig,
}

impl ProbabilisticScorer {
    /// Create a new probabilistic scorer with configuration
    pub fn new(config: MatchingConfig) -> Self {
        Self { config }
    }

    /// Calculate match score between two patients
    pub fn calculate_score(
        &self,
        patient: &Patient,
        candidate: &Patient,
    ) -> MatchResult {
        // Weight factors for each component
        const NAME_WEIGHT: f64 = 0.35;
        const DOB_WEIGHT: f64 = 0.30;
        const GENDER_WEIGHT: f64 = 0.10;
        const ADDRESS_WEIGHT: f64 = 0.15;
        const IDENTIFIER_WEIGHT: f64 = 0.10;

        // Calculate individual component scores
        let name_score = name_matching::match_names(&patient.name, &candidate.name);

        let birth_date_score = dob_matching::match_birth_dates(
            patient.birth_date,
            candidate.birth_date,
        );

        let gender_score = gender_matching::match_gender(
            patient.gender,
            candidate.gender,
        );

        let address_score = address_matching::match_addresses(
            &patient.addresses,
            &candidate.addresses,
        );

        let identifier_score = identifier_matching::match_identifiers(
            &patient.identifiers,
            &candidate.identifiers,
        );

        // Calculate weighted total score
        let total_score = (name_score * NAME_WEIGHT)
            + (birth_date_score * DOB_WEIGHT)
            + (gender_score * GENDER_WEIGHT)
            + (address_score * ADDRESS_WEIGHT)
            + (identifier_score * IDENTIFIER_WEIGHT);

        let breakdown = MatchScoreBreakdown {
            name_score,
            birth_date_score,
            gender_score,
            address_score,
            identifier_score,
        };

        MatchResult {
            patient: candidate.clone(),
            score: total_score,
            breakdown,
        }
    }

    /// Check if a match score meets the threshold
    pub fn is_match(&self, score: f64) -> bool {
        score >= self.config.threshold_score
    }

    /// Classify match quality
    pub fn classify_match(&self, score: f64) -> MatchQuality {
        if score >= 0.95 {
            MatchQuality::Definite
        } else if score >= self.config.threshold_score {
            MatchQuality::Probable
        } else if score >= 0.50 {
            MatchQuality::Possible
        } else {
            MatchQuality::Unlikely
        }
    }
}

/// Deterministic scoring strategy
pub struct DeterministicScorer {
    /// Configuration for matching
    config: MatchingConfig,
}

impl DeterministicScorer {
    /// Create a new deterministic scorer
    pub fn new(config: MatchingConfig) -> Self {
        Self { config }
    }

    /// Calculate match score using strict rules
    pub fn calculate_score(
        &self,
        patient: &Patient,
        candidate: &Patient,
    ) -> MatchResult {
        let mut total_score = 0.0;
        let mut points_available = 0.0;

        // Rule 1: Exact identifier match = definite match
        let identifier_score = identifier_matching::match_identifiers(
            &patient.identifiers,
            &candidate.identifiers,
        );

        if identifier_score >= 0.98 {
            // Exact identifier match - return definite match
            return MatchResult {
                patient: candidate.clone(),
                score: 1.0,
                breakdown: MatchScoreBreakdown {
                    name_score: 0.0,
                    birth_date_score: 0.0,
                    gender_score: 0.0,
                    address_score: 0.0,
                    identifier_score,
                },
            };
        }

        // Rule 2: Name + DOB + Gender must all match
        let name_score = name_matching::match_names(&patient.name, &candidate.name);
        let dob_score = dob_matching::match_birth_dates(
            patient.birth_date,
            candidate.birth_date,
        );
        let gender_score = gender_matching::match_gender(
            patient.gender,
            candidate.gender,
        );

        points_available += 3.0;

        if name_score >= 0.90 {
            total_score += 1.0;
        }

        if dob_score >= 0.95 {
            total_score += 1.0;
        }

        if gender_score >= 1.0 {
            total_score += 1.0;
        }

        // Rule 3: Address is optional but adds confidence
        let address_score = address_matching::match_addresses(
            &patient.addresses,
            &candidate.addresses,
        );

        if !patient.addresses.is_empty() && !candidate.addresses.is_empty() {
            points_available += 1.0;
            if address_score >= 0.80 {
                total_score += 1.0;
            }
        }

        // Calculate final score as percentage of available points
        let final_score = if points_available > 0.0 {
            total_score / points_available
        } else {
            0.0
        };

        let breakdown = MatchScoreBreakdown {
            name_score,
            birth_date_score: dob_score,
            gender_score,
            address_score,
            identifier_score,
        };

        MatchResult {
            patient: candidate.clone(),
            score: final_score,
            breakdown,
        }
    }

    /// Check if a match score meets deterministic criteria
    pub fn is_match(&self, score: f64) -> bool {
        score >= 0.75 // Require at least 3/4 rules to match
    }
}

/// Match quality classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchQuality {
    /// Definite match (score >= 0.95)
    Definite,
    /// Probable match (score >= threshold)
    Probable,
    /// Possible match (score >= 0.50)
    Possible,
    /// Unlikely match (score < 0.50)
    Unlikely,
}

impl MatchQuality {
    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            MatchQuality::Definite => "definite",
            MatchQuality::Probable => "probable",
            MatchQuality::Possible => "possible",
            MatchQuality::Unlikely => "unlikely",
        }
    }

    /// Check if this quality indicates a match
    pub fn is_match(&self) -> bool {
        matches!(self, MatchQuality::Definite | MatchQuality::Probable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{HumanName, Gender};
    use chrono::NaiveDate;

    fn create_test_config() -> MatchingConfig {
        MatchingConfig {
            threshold_score: 0.85,
            exact_match_score: 1.0,
            fuzzy_match_score: 0.8,
        }
    }

    fn create_test_patient(name: &str, dob: Option<NaiveDate>) -> Patient {
        Patient {
            id: uuid::Uuid::new_v4(),
            identifiers: vec![],
            active: true,
            name: HumanName {
                use_type: None,
                family: name.to_string(),
                given: vec!["John".to_string()],
                prefix: vec![],
                suffix: vec![],
            },
            additional_names: vec![],
            telecom: vec![],
            gender: Gender::Male,
            birth_date: dob,
            deceased: false,
            deceased_datetime: None,
            addresses: vec![],
            marital_status: None,
            multiple_birth: None,
            photo: vec![],
            managing_organization: None,
            links: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_exact_match_scores_high() {
        let config = create_test_config();
        let scorer = ProbabilisticScorer::new(config);

        let dob = NaiveDate::from_ymd_opt(1980, 1, 15);
        let patient1 = create_test_patient("Smith", dob);
        let patient2 = create_test_patient("Smith", dob);

        let result = scorer.calculate_score(&patient1, &patient2);

        // With NAME (0.35) + DOB (0.30) + GENDER (0.10) = 0.75
        // No address or identifiers, so those contribute 0
        assert!(result.score >= 0.70, "Exact match on name/dob/gender should score >= 0.70, got {}", result.score);
        assert!(!scorer.is_match(result.score)); // 0.75 < threshold of 0.85
        assert_eq!(scorer.classify_match(result.score), MatchQuality::Possible);
    }

    #[test]
    fn test_fuzzy_match_scores_moderate() {
        let config = create_test_config();
        let scorer = ProbabilisticScorer::new(config);

        let dob1 = NaiveDate::from_ymd_opt(1980, 1, 15);
        let dob2 = NaiveDate::from_ymd_opt(1980, 1, 16); // One day off

        let patient1 = create_test_patient("Smith", dob1);
        let patient2 = create_test_patient("Smyth", dob2); // Spelling variant

        let result = scorer.calculate_score(&patient1, &patient2);

        assert!(result.score > 0.60, "Fuzzy match should score > 0.60, got {}", result.score);
        assert!(result.score < 0.80);
    }

    #[test]
    fn test_no_match_scores_low() {
        let config = create_test_config();
        let scorer = ProbabilisticScorer::new(config);

        let dob1 = NaiveDate::from_ymd_opt(1980, 1, 15);
        let dob2 = NaiveDate::from_ymd_opt(1990, 6, 20);

        let patient1 = create_test_patient("Smith", dob1);
        let patient2 = create_test_patient("Johnson", dob2);

        let result = scorer.calculate_score(&patient1, &patient2);

        assert!(result.score < 0.50, "Non-match should score < 0.50, got {}", result.score);
        assert!(!scorer.is_match(result.score));
    }

    #[test]
    fn test_deterministic_exact_match() {
        let config = create_test_config();
        let scorer = DeterministicScorer::new(config);

        let dob = NaiveDate::from_ymd_opt(1980, 1, 15);
        let patient1 = create_test_patient("Smith", dob);
        let patient2 = create_test_patient("Smith", dob);

        let result = scorer.calculate_score(&patient1, &patient2);

        assert!(result.score >= 0.75, "Exact match should meet deterministic threshold");
        assert!(scorer.is_match(result.score));
    }

    #[test]
    fn test_match_quality_classification() {
        assert_eq!(ProbabilisticScorer::new(create_test_config())
            .classify_match(0.98), MatchQuality::Definite);

        assert_eq!(ProbabilisticScorer::new(create_test_config())
            .classify_match(0.87), MatchQuality::Probable);

        assert_eq!(ProbabilisticScorer::new(create_test_config())
            .classify_match(0.60), MatchQuality::Possible);

        assert_eq!(ProbabilisticScorer::new(create_test_config())
            .classify_match(0.30), MatchQuality::Unlikely);
    }
}
