//! Shared path-safety predicates for quarantine and removal plans. Both
//! features call these, so the blocklists cannot diverge again.

use std::path::{Path, PathBuf};

/// Home directory from `HOME` then `USERPROFILE`; relative values are
/// ignored, never returned.
pub(crate) fn home_dir() -> Option<PathBuf> {
    for var in ["HOME", "USERPROFILE"] {
        if let Some(value) = std::env::var_os(var) {
            let p = PathBuf::from(value);
            if p.is_absolute() {
                return Some(p);
            }
        }
    }
    None
}

/// Whether `path` carries glob metacharacters (`*`, `?`, `[`).
pub(crate) fn has_glob(path: &Path) -> bool {
    let s = path.to_string_lossy();
    s.contains('*') || s.contains('?') || s.contains('[')
}

/// Whether `path` still holds an unresolved `$`/`%` variable, or a
/// whole-component `~` (unexpanded home shorthand; Windows 8.3 short names
/// like `RUNNER~1` are legal and pass).
pub(crate) fn has_unresolved_variable(path: &Path) -> bool {
    let s = path.to_string_lossy();
    if s.contains('$') || s.contains('%') {
        return true;
    }
    path.components().any(|c| c.as_os_str() == "~")
}

/// Broad roots no quarantine or removal may target: the unix system roots in
/// bare and trailing-slash spellings, plus the Windows-shaped roots
/// ([`crate::transaction::windows_shaped_broad_root`]), which are inert on
/// unix hosts.
pub(crate) fn is_broad_root(path: &Path) -> bool {
    if crate::transaction::windows_shaped_broad_root(path) {
        return true;
    }
    let s = path.to_string_lossy();
    matches!(
        s.as_ref(),
        "/" | "/home"
            | "/home/"
            | "/tmp"
            | "/tmp/"
            | "/usr"
            | "/usr/"
            | "/etc"
            | "/etc/"
            | "/var"
            | "/var/"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broad_root_matrix_covers_bare_and_trailing_slash_spellings() {
        let refused = [
            "/", "/home", "/home/", "/tmp", "/tmp/", "/usr", "/usr/", "/etc", "/etc/", "/var",
            "/var/",
        ];
        for p in refused {
            assert!(is_broad_root(Path::new(p)), "{p} must be refused");
        }
        // Children and lookalikes stay allowed; only exact roots are broad.
        let allowed = [
            "/tmp/x",
            "/home/me",
            "/usr/local",
            "/etc/hosts",
            "/var/log",
            "/hostname",
            "/usrt",
            "/Home",
        ];
        for p in allowed {
            assert!(!is_broad_root(Path::new(p)), "{p} must not be refused");
        }
        // Windows-shaped roots are broad on every host; forward-slash UNC
        // counts only on Windows.
        assert!(is_broad_root(Path::new("C:\\")));
        assert_eq!(is_broad_root(Path::new("//server/share")), cfg!(windows));
    }

    #[test]
    fn glob_and_variable_checks_match_the_dangerous_shapes() {
        for p in ["a*b", "a?b", "a[b", "a$", "a%", "/home/~"] {
            assert!(
                has_glob(Path::new(p)) || has_unresolved_variable(Path::new(p)),
                "{p} must be flagged"
            );
        }
        for p in ["/tmp/ok", "/home/u/tool", "C:\\Users\\u\\RUNNER~1\\x"] {
            assert!(
                !has_glob(Path::new(p)) && !has_unresolved_variable(Path::new(p)),
                "{p} must pass"
            );
        }
    }
}
