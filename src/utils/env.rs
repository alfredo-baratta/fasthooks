//! Environment detection utilities

use std::env;

/// Check if running in a CI environment
#[allow(dead_code)]
pub fn is_ci() -> bool {
    // Common CI environment variables
    const CI_VARS: &[&str] = &[
        "CI",
        "CONTINUOUS_INTEGRATION",
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "CIRCLECI",
        "TRAVIS",
        "JENKINS_URL",
        "BUILDKITE",
        "DRONE",
        "AZURE_PIPELINES",
        "TEAMCITY_VERSION",
    ];

    CI_VARS.iter().any(|var| env::var(var).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ci_detection() {
        // In normal test environment, CI might or might not be set
        // Just verify the function doesn't panic
        let _ = is_ci();
    }
}
