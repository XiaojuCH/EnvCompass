use std::str::FromStr;

use node_semver::{Range, Version as NodeVersion};
use pep440_rs::{Version as PythonVersion, VersionSpecifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequirementSatisfaction {
    Satisfied,
    NotSatisfied,
    Unsupported,
}

impl RequirementSatisfaction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Satisfied => "satisfied",
            Self::NotSatisfied => "not_satisfied",
            Self::Unsupported => "unknown",
        }
    }
}

pub fn python_satisfies(requirement: &str, actual: &str) -> RequirementSatisfaction {
    let requirement = requirement.trim();
    let actual = actual.trim();
    if requirement.is_empty() || actual.is_empty() {
        return RequirementSatisfaction::Unsupported;
    }

    let Ok(specifiers) = VersionSpecifiers::from_str(requirement) else {
        return RequirementSatisfaction::Unsupported;
    };
    let Ok(version) = PythonVersion::from_str(actual) else {
        return RequirementSatisfaction::Unsupported;
    };

    if specifiers.contains(&version) {
        RequirementSatisfaction::Satisfied
    } else {
        RequirementSatisfaction::NotSatisfied
    }
}

pub fn node_satisfies(requirement: &str, actual: &str) -> RequirementSatisfaction {
    let requirement = requirement.trim();
    let actual = actual.trim().trim_start_matches('v');
    if requirement.is_empty() || actual.is_empty() {
        return RequirementSatisfaction::Unsupported;
    }

    let Ok(range) = Range::parse(requirement) else {
        return RequirementSatisfaction::Unsupported;
    };
    let Ok(version) = NodeVersion::parse(actual) else {
        return RequirementSatisfaction::Unsupported;
    };

    if range.satisfies(&version) {
        RequirementSatisfaction::Satisfied
    } else {
        RequirementSatisfaction::NotSatisfied
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn python_exact_range_avoids_string_comparison_bug() {
        assert_eq!(
            python_satisfies(">=3.11", "3.9.0"),
            RequirementSatisfaction::NotSatisfied
        );
        assert_eq!(
            python_satisfies(">=3.11", "3.12.0"),
            RequirementSatisfaction::Satisfied
        );
    }

    #[test]
    fn python_requires_python_common_ranges() {
        assert_eq!(
            python_satisfies(">=3.8,<4", "3.12.10"),
            RequirementSatisfaction::Satisfied
        );
        assert_eq!(
            python_satisfies("~=3.11.0", "3.11.4"),
            RequirementSatisfaction::Satisfied
        );
        assert_eq!(
            python_satisfies("~=3.11.0", "3.12.0"),
            RequirementSatisfaction::NotSatisfied
        );
        assert_eq!(
            python_satisfies("==3.11.*", "3.11.9"),
            RequirementSatisfaction::Satisfied
        );
    }

    #[test]
    fn node_semver_ranges() {
        assert_eq!(
            node_satisfies(">=20 <23", "24.0.0"),
            RequirementSatisfaction::NotSatisfied
        );
        assert_eq!(
            node_satisfies(">=20 <23", "22.23.1"),
            RequirementSatisfaction::Satisfied
        );
        assert_eq!(
            node_satisfies("^20.0.0", "20.19.0"),
            RequirementSatisfaction::Satisfied
        );
        assert_eq!(
            node_satisfies("20.x", "20.11.0"),
            RequirementSatisfaction::Satisfied
        );
    }

    #[test]
    fn malformed_requirements_are_unknown() {
        assert_eq!(
            python_satisfies("not-a-specifier", "3.12.0"),
            RequirementSatisfaction::Unsupported
        );
        assert_eq!(
            node_satisfies("garbage", "22.0.0"),
            RequirementSatisfaction::Unsupported
        );
    }
}

