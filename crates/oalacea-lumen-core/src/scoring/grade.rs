//! Grade system with A+/A/A-/B+/B/B-/C+/C/C-/D+/D/F scale

use serde::{Deserialize, Serialize};

/// Grade representing score quality
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    APlus,   // 97-100: Outstanding
    A,       // 93-96: Excellent
    AMinus,  // 90-92: Very Good
    BPlus,   // 87-89: Good
    B,       // 83-86: Above Average
    BMinus,  // 80-82: Slightly Above Average
    CPlus,   // 77-79: Average
    C,       // 73-76: Slightly Below Average
    CMinus,  // 70-72: Below Average
    DPlus,   // 60-69: Poor
    D,       // 50-59: Very Poor
    F,       // 0-49: Failing
}

impl Grade {
    /// Convert a numeric score (0-100) to a Grade
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 97.0 => Grade::APlus,
            s if s >= 93.0 => Grade::A,
            s if s >= 90.0 => Grade::AMinus,
            s if s >= 87.0 => Grade::BPlus,
            s if s >= 83.0 => Grade::B,
            s if s >= 80.0 => Grade::BMinus,
            s if s >= 77.0 => Grade::CPlus,
            s if s >= 73.0 => Grade::C,
            s if s >= 70.0 => Grade::CMinus,
            s if s >= 60.0 => Grade::DPlus,
            s if s >= 50.0 => Grade::D,
            _ => Grade::F,
        }
    }

    /// Get the letter representation (e.g., "A+", "B", "F")
    pub fn as_letter(&self) -> &str {
        match self {
            Grade::APlus => "A+",
            Grade::A => "A",
            Grade::AMinus => "A-",
            Grade::BPlus => "B+",
            Grade::B => "B",
            Grade::BMinus => "B-",
            Grade::CPlus => "C+",
            Grade::C => "C",
            Grade::CMinus => "C-",
            Grade::DPlus => "D+",
            Grade::D => "D",
            Grade::F => "F",
        }
    }

    /// Get the descriptive label
    pub fn label(&self) -> &str {
        match self {
            Grade::APlus => "Outstanding - Exceptionnel",
            Grade::A => "Excellent - Excellent",
            Grade::AMinus => "Very Good - Très bon",
            Grade::BPlus => "Good - Bon",
            Grade::B => "Above Average - Au-dessus de la moyenne",
            Grade::BMinus => "Slightly Above Average - Légèrement au-dessus",
            Grade::CPlus => "Average - Moyen",
            Grade::C => "Slightly Below Average - Légèrement en dessous",
            Grade::CMinus => "Below Average - En dessous de la moyenne",
            Grade::DPlus => "Poor - Médiocre",
            Grade::D => "Very Poor - Très médiocre",
            Grade::F => "Failing - Échec",
        }
    }

    /// Get the ANSI color code for terminal display
    pub fn ansi_color(&self) -> &str {
        match self {
            Grade::APlus | Grade::A | Grade::AMinus => "\x1b[32m", // green
            Grade::BPlus | Grade::B | Grade::BMinus => "\x1b[34m", // blue
            Grade::CPlus | Grade::C | Grade::CMinus => "\x1b[33m", // yellow
            Grade::DPlus | Grade::D => "\x1b[31m", // orange/red
            Grade::F => "\x1b[31m", // red
        }
    }

    /// Get CSS color for web display
    pub fn css_color(&self) -> &str {
        match self {
            Grade::APlus | Grade::A | Grade::AMinus => "#10b981", // green-500
            Grade::BPlus | Grade::B | Grade::BMinus => "#3b82f6", // blue-500
            Grade::CPlus | Grade::C | Grade::CMinus => "#f59e0b", // amber-500
            Grade::DPlus | Grade::D => "#f97316", // orange-500
            Grade::F => "#ef4444", // red-500
        }
    }

    /// Get the GPA value on a 4.0 scale
    pub fn gpa_value(&self) -> f64 {
        match self {
            Grade::APlus => 4.0,
            Grade::A => 4.0,
            Grade::AMinus => 3.7,
            Grade::BPlus => 3.3,
            Grade::B => 3.0,
            Grade::BMinus => 2.7,
            Grade::CPlus => 2.3,
            Grade::C => 2.0,
            Grade::CMinus => 1.7,
            Grade::DPlus => 1.3,
            Grade::D => 1.0,
            Grade::F => 0.0,
        }
    }

    /// Get the numeric score range for this grade
    pub fn score_range(&self) -> (f64, f64) {
        match self {
            Grade::APlus => (97.0, 100.0),
            Grade::A => (93.0, 96.99),
            Grade::AMinus => (90.0, 92.99),
            Grade::BPlus => (87.0, 89.99),
            Grade::B => (83.0, 86.99),
            Grade::BMinus => (80.0, 82.99),
            Grade::CPlus => (77.0, 79.99),
            Grade::C => (73.0, 76.99),
            Grade::CMinus => (70.0, 72.99),
            Grade::DPlus => (60.0, 69.99),
            Grade::D => (50.0, 59.99),
            Grade::F => (0.0, 49.99),
        }
    }

    /// Check if this is a passing grade
    pub fn is_passing(&self) -> bool {
        !matches!(self, Grade::F)
    }

    /// Check if this is a good grade (B- or above)
    pub fn is_good(&self) -> bool {
        self.gpa_value() >= 2.7
    }

    /// Check if this is an excellent grade (A- or above)
    pub fn is_excellent(&self) -> bool {
        self.gpa_value() >= 3.7
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_letter())
    }
}

/// Grade system with configuration for thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeSystem {
    /// Minimum score for A+
    pub a_plus_min: f64,
    /// Minimum score for A
    pub a_min: f64,
    /// Minimum score for A-
    pub a_minus_min: f64,
    /// Minimum score for B+
    pub b_plus_min: f64,
    /// Minimum score for B
    pub b_min: f64,
    /// Minimum score for B-
    pub b_minus_min: f64,
    /// Minimum score for C+
    pub c_plus_min: f64,
    /// Minimum score for C
    pub c_min: f64,
    /// Minimum score for C-
    pub c_minus_min: f64,
    /// Minimum score for D+
    pub d_plus_min: f64,
    /// Minimum score for D
    pub d_min: f64,
}

impl Default for GradeSystem {
    fn default() -> Self {
        Self {
            a_plus_min: 97.0,
            a_min: 93.0,
            a_minus_min: 90.0,
            b_plus_min: 87.0,
            b_min: 83.0,
            b_minus_min: 80.0,
            c_plus_min: 77.0,
            c_min: 73.0,
            c_minus_min: 70.0,
            d_plus_min: 60.0,
            d_min: 50.0,
        }
    }
}

impl GradeSystem {
    /// Convert a numeric score to Grade using this system's thresholds
    pub fn grade_from_score(&self, score: f64) -> Grade {
        match score {
            s if s >= self.a_plus_min => Grade::APlus,
            s if s >= self.a_min => Grade::A,
            s if s >= self.a_minus_min => Grade::AMinus,
            s if s >= self.b_plus_min => Grade::BPlus,
            s if s >= self.b_min => Grade::B,
            s if s >= self.b_minus_min => Grade::BMinus,
            s if s >= self.c_plus_min => Grade::CPlus,
            s if s >= self.c_min => Grade::C,
            s if s >= self.c_minus_min => Grade::CMinus,
            s if s >= self.d_plus_min => Grade::DPlus,
            s if s >= self.d_min => Grade::D,
            _ => Grade::F,
        }
    }

    /// Create a stricter grading system
    pub fn strict() -> Self {
        Self {
            a_plus_min: 98.0,
            a_min: 95.0,
            a_minus_min: 92.0,
            b_plus_min: 88.0,
            b_min: 85.0,
            b_minus_min: 82.0,
            c_plus_min: 78.0,
            c_min: 75.0,
            c_minus_min: 72.0,
            d_plus_min: 65.0,
            d_min: 55.0,
        }
    }

    /// Create a more lenient grading system
    pub fn lenient() -> Self {
        Self {
            a_plus_min: 95.0,
            a_min: 90.0,
            a_minus_min: 85.0,
            b_plus_min: 80.0,
            b_min: 75.0,
            b_minus_min: 70.0,
            c_plus_min: 65.0,
            c_min: 60.0,
            c_minus_min: 55.0,
            d_plus_min: 50.0,
            d_min: 40.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_from_score() {
        assert_eq!(Grade::from_score(98.0), Grade::APlus);
        assert_eq!(Grade::from_score(94.0), Grade::A);
        assert_eq!(Grade::from_score(91.0), Grade::AMinus);
        assert_eq!(Grade::from_score(88.0), Grade::BPlus);
        assert_eq!(Grade::from_score(84.0), Grade::B);
        assert_eq!(Grade::from_score(81.0), Grade::BMinus);
        assert_eq!(Grade::from_score(78.0), Grade::CPlus);
        assert_eq!(Grade::from_score(74.0), Grade::C);
        assert_eq!(Grade::from_score(71.0), Grade::CMinus);
        assert_eq!(Grade::from_score(65.0), Grade::DPlus);
        assert_eq!(Grade::from_score(55.0), Grade::D);
        assert_eq!(Grade::from_score(30.0), Grade::F);
    }

    #[test]
    fn test_gpa_values() {
        assert_eq!(Grade::APlus.gpa_value(), 4.0);
        assert_eq!(Grade::AMinus.gpa_value(), 3.7);
        assert_eq!(Grade::B.gpa_value(), 3.0);
        assert_eq!(Grade::C.gpa_value(), 2.0);
        assert_eq!(Grade::D.gpa_value(), 1.0);
        assert_eq!(Grade::F.gpa_value(), 0.0);
    }

    #[test]
    fn test_is_passing() {
        assert!(Grade::A.is_passing());
        assert!(Grade::B.is_passing());
        assert!(Grade::C.is_passing());
        assert!(Grade::D.is_passing());
        assert!(!Grade::F.is_passing());
    }

    #[test]
    fn test_is_good() {
        assert!(Grade::A.is_good());
        assert!(Grade::AMinus.is_good());
        assert!(Grade::BMinus.is_good()); // B- has GPA 2.7, which is >= 2.7
        assert!(!Grade::C.is_good()); // C has GPA 2.0, which is < 2.7
    }

    #[test]
    fn test_custom_grade_system() {
        let strict = GradeSystem::strict();
        assert_eq!(strict.grade_from_score(98.0), Grade::APlus);
        assert_eq!(strict.grade_from_score(95.0), Grade::A);
        assert_eq!(strict.grade_from_score(92.0), Grade::AMinus);

        let lenient = GradeSystem::lenient();
        assert_eq!(lenient.grade_from_score(95.0), Grade::APlus);
        assert_eq!(lenient.grade_from_score(90.0), Grade::A);
        assert_eq!(lenient.grade_from_score(85.0), Grade::AMinus);
        assert_eq!(lenient.grade_from_score(72.0), Grade::BMinus); // 72 >= 70 (b_minus_min)
        assert_eq!(lenient.grade_from_score(67.0), Grade::CPlus); // 67 >= 65 (c_plus_min)
    }
}
