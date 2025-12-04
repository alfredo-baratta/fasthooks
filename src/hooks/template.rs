//! Git hook script templates

use crate::config::HookType;

/// Generates hook script content
pub struct HookTemplate;

impl HookTemplate {
    /// Generate a hook script for the given hook type
    pub fn generate(hook_type: HookType) -> String {
        let hook_name = hook_type.as_str();

        format!(
            r#"#!/bin/sh
# FastHooks - https://github.com/alfredo-baratta/fasthooks
# This hook was automatically generated. Do not edit.
# Hook: {hook_name}

# Exit on error
set -e

# Check if fasthooks is available
if ! command -v fasthooks &> /dev/null; then
    echo "fasthooks: command not found"
    echo "Please install fasthooks or add it to your PATH"
    echo "Install: cargo install fasthooks"
    exit 1
fi

# Run the hook
fasthooks run {hook_name} "$@"
exit_code=$?

exit $exit_code
"#,
            hook_name = hook_name
        )
    }

    /// Generate a Windows batch file hook
    #[allow(dead_code)]
    pub fn generate_windows(hook_type: HookType) -> String {
        let hook_name = hook_type.as_str();

        format!(
            r#"@echo off
REM FastHooks - https://github.com/alfredo-baratta/fasthooks
REM This hook was automatically generated. Do not edit.
REM Hook: {hook_name}

where fasthooks >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo fasthooks: command not found
    echo Please install fasthooks or add it to your PATH
    echo Install: cargo install fasthooks
    exit /b 1
)

fasthooks run {hook_name} %*
exit /b %ERRORLEVEL%
"#,
            hook_name = hook_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hook() {
        let script = HookTemplate::generate(HookType::PreCommit);
        assert!(script.contains("fasthooks"));
        assert!(script.contains("pre-commit"));
        assert!(script.contains("#!/bin/sh"));
    }

    #[test]
    fn test_generate_windows_hook() {
        let script = HookTemplate::generate_windows(HookType::PreCommit);
        assert!(script.contains("fasthooks"));
        assert!(script.contains("pre-commit"));
        assert!(script.contains("@echo off"));
    }
}
