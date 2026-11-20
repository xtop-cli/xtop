//! macOS Directory Services user probe via `/usr/bin/dscl`.
//!
//! `/etc/passwd` on macOS only lists kernel/system accounts; the interactive
//! users live in the Open Directory database. This probe fetches them once
//! and the kernel merges the result into the uid table alongside the
//! `passwd(5)` entries (see `crate::core::state::users`).

use std::process::Command;

/// Login name and uid pairs from Directory Services.
///
/// Output shape per line: `username<whitespace>uid`. System accounts whose
/// name starts with `_` are filtered out — they are not interesting for the
/// processes view and are mostly absent from `/etc/passwd` on macOS.
pub fn read_directory_users() -> Vec<(u32, String)> {
    let output = match Command::new("/usr/bin/dscl")
        .args([".", "-list", "/Users", "UniqueID"])
        .env("LC_ALL", "C")
        .output()
    {
        Ok(out) if out.status.success() => out.stdout,
        _ => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output);
    parse_dscl(&text)
}

fn parse_dscl(output: &str) -> Vec<(u32, String)> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let name = fields.next()?;
            let uid = fields.next()?.parse::<u32>().ok()?;
            if name.starts_with('_') || name.is_empty() {
                return None;
            }
            Some((uid, name.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_lines_and_skips_system_accounts() {
        let text = "_accessoryupdater\t278\n\
            root\t0\n\
            xscriptor\t501\n\
            daemon\t1\n\
            broken\n\
            emptyuid\tnotanumber\n";
        let users = parse_dscl(text);
        assert!(users.contains(&(501, "xscriptor".to_string())));
        assert!(users.contains(&(0, "root".to_string())));
        assert!(users.contains(&(1, "daemon".to_string())));
        assert!(!users.iter().any(|(_, name)| name.starts_with('_')));
        assert_eq!(users.len(), 3);
    }

    #[test]
    fn empty_output_yields_empty_list() {
        assert!(parse_dscl("").is_empty());
    }
}
