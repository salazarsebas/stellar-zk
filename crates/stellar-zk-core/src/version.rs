//! Version parsing and detection for external tool prerequisites.
//!
//! Provides lightweight version comparison without adding dependencies.
//! If a tool doesn't support `--version` or produces unexpected output,
//! detection silently returns `None` — this is intentional (warnings, not errors).

use std::fmt;
use std::process::Command;

/// A semver-like version with major.minor.patch components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Parse the first `X.Y.Z` pattern found in a string.
    ///
    /// Handles common formats:
    /// - `"2.1.8"`
    /// - `"v0.36.0"`
    /// - `"nargo version = 0.36.0"`
    /// - `"snarkjs@0.7.4"`
    pub fn parse(s: &str) -> Option<Self> {
        // Find the first digit that starts an X.Y.Z pattern
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i].is_ascii_digit() {
                // Try to parse X.Y.Z starting here
                if let Some((ver, _)) = Self::parse_at(s, i) {
                    return Some(ver);
                }
            }
            i += 1;
        }
        None
    }

    /// Try to parse `X.Y.Z` starting at byte offset `start`.
    /// Returns the version and the byte offset after the last digit.
    fn parse_at(s: &str, start: usize) -> Option<(Self, usize)> {
        let rest = &s[start..];
        let mut parts = rest.splitn(4, '.');
        let major_str = parts.next()?;
        let minor_str = parts.next()?;
        let patch_part = parts.next()?;

        let major: u32 = major_str.parse().ok()?;
        let minor: u32 = minor_str.parse().ok()?;
        // patch_part may have trailing non-digit chars (e.g., "0-beta1")
        let patch_str: String = patch_part
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if patch_str.is_empty() {
            return None;
        }
        let patch: u32 = patch_str.parse().ok()?;

        Some((
            Self {
                major,
                minor,
                patch,
            },
            start + major_str.len() + 1 + minor_str.len() + 1 + patch_str.len(),
        ))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Run `tool --version` and parse the output.
///
/// Returns `None` if the tool is not found, exits with error, or
/// produces output that doesn't contain an `X.Y.Z` pattern.
pub fn detect_version(tool: &str) -> Option<Version> {
    let output = Command::new(tool).arg("--version").output().ok()?;

    if !output.status.success() {
        // Some tools print version to stderr, try both
        let stderr = String::from_utf8_lossy(&output.stderr);
        if let Some(v) = Version::parse(&stderr) {
            return Some(v);
        }
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(v) = Version::parse(&stdout) {
        return Some(v);
    }

    // Fallback: try stderr (some tools print version info there)
    let stderr = String::from_utf8_lossy(&output.stderr);
    Version::parse(&stderr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let v = Version::parse("2.1.8").unwrap();
        assert_eq!(
            v,
            Version {
                major: 2,
                minor: 1,
                patch: 8
            }
        );
    }

    #[test]
    fn test_parse_with_v_prefix() {
        let v = Version::parse("v0.36.0").unwrap();
        assert_eq!(
            v,
            Version {
                major: 0,
                minor: 36,
                patch: 0
            }
        );
    }

    #[test]
    fn test_parse_nargo_format() {
        let v = Version::parse("nargo version = 0.36.0").unwrap();
        assert_eq!(
            v,
            Version {
                major: 0,
                minor: 36,
                patch: 0
            }
        );
    }

    #[test]
    fn test_parse_node_format() {
        let v = Version::parse("v20.11.1").unwrap();
        assert_eq!(
            v,
            Version {
                major: 20,
                minor: 11,
                patch: 1
            }
        );
    }

    #[test]
    fn test_parse_invalid() {
        assert!(Version::parse("no version here").is_none());
        assert!(Version::parse("").is_none());
        assert!(Version::parse("1.2").is_none());
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version {
            major: 1,
            minor: 0,
            patch: 0,
        };
        let v2 = Version {
            major: 2,
            minor: 0,
            patch: 0,
        };
        let v3 = Version {
            major: 1,
            minor: 1,
            patch: 0,
        };
        let v4 = Version {
            major: 1,
            minor: 0,
            patch: 1,
        };

        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v1 < v4);
        assert!(v3 < v2);
    }

    #[test]
    fn test_detect_version_nonexistent_tool() {
        assert!(detect_version("this_tool_does_not_exist_xyz").is_none());
    }

    #[test]
    fn test_version_display() {
        let v = Version {
            major: 1,
            minor: 2,
            patch: 3,
        };
        assert_eq!(v.to_string(), "1.2.3");
    }
}
