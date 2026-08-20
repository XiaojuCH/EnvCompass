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

    let normalized = if let Some(poetry_range) = poetry_python_range(requirement) {
        poetry_range
    } else if requirement
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit() || c == 'v')
    {
        let core = requirement.trim_start_matches('v');
        if !core.contains('*') && core.split('.').count() <= 2 {
            format!("=={core}.*")
        } else {
            format!("=={core}")
        }
    } else {
        requirement.to_string()
    };

    let Ok(specifiers) = VersionSpecifiers::from_str(&normalized) else {
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

fn poetry_python_range(requirement: &str) -> Option<String> {
    let (operator, version) = if let Some(version) = requirement.strip_prefix('^') {
        ('^', version)
    } else if requirement.starts_with('~') && !requirement.starts_with("~=") {
        ('~', requirement.trim_start_matches('~'))
    } else {
        return None;
    };
    let parts = version
        .split('.')
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }

    let upper = if operator == '^' {
        let first_nonzero = parts
            .iter()
            .position(|part| *part != 0)
            .unwrap_or(parts.len() - 1);
        let mut upper = parts.clone();
        upper[first_nonzero] += 1;
        upper.truncate(first_nonzero + 1);
        while upper.len() < 2 {
            upper.push(0);
        }
        upper
    } else if parts.len() == 1 {
        vec![parts[0] + 1, 0]
    } else {
        vec![parts[0], parts[1] + 1]
    };

    Some(format!(
        ">={version},<{}",
        upper
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(".")
    ))
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
        assert_eq!(
            python_satisfies("3.11", "3.12.10"),
            RequirementSatisfaction::NotSatisfied
        );
        assert_eq!(
            python_satisfies("3.11", "3.11.9"),
            RequirementSatisfaction::Satisfied
        );
        assert_eq!(
            python_satisfies("^3.10", "3.12.0"),
            RequirementSatisfaction::Satisfied
        );
        assert_eq!(
            python_satisfies("^3.10", "4.0.0"),
            RequirementSatisfaction::NotSatisfied
        );
        assert_eq!(
            python_satisfies("^0.0", "0.0.9"),
            RequirementSatisfaction::Satisfied
        );
        assert_eq!(
            python_satisfies("^0.0", "0.1.0"),
            RequirementSatisfaction::NotSatisfied
        );
        assert_eq!(
            python_satisfies("~3.10", "3.11.0"),
            RequirementSatisfaction::NotSatisfied
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
