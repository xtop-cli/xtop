//! Windows user probes.
//!
//! sysinfo reports Windows user ids as SIDs (`S-1-5-21-...-1000`), but the
//! kernel's uid table and the process-widget `uid_to_name` resolution only
//! match numeric ids, so the trailing RID is exposed (see
//! [`read_process_user_id`]).
//!
//! The RID → login-name table comes from the local accounts, queried once
//! at startup through `Get-LocalUser` (JSON output, parsed purely).

use std::process::Command;

const LOCAL_USERS_SCRIPT: &str = "Get-LocalUser | ForEach-Object { [pscustomobject]@{ rid = [int]($_.SID.Value.Split('-')[-1]); name = $_.Name } } | ConvertTo-Json -Compress";

/// Local account RID → name pairs via `Get-LocalUser`, resolved once at
/// startup by the kernel (`Users::load`); no PowerShell on the system →
/// empty table and the numeric uid fallback applies.
pub fn read_directory_users() -> Vec<(u32, String)> {
    let output = match Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            LOCAL_USERS_SCRIPT,
        ])
        .output()
    {
        Ok(out) if out.status.success() => out.stdout,
        _ => return Vec::new(),
    };
    parse_local_users(&String::from_utf8_lossy(&output))
}

/// The trailing numeric RID of a SID string (`S-1-5-21-...-1000` → `1000`),
/// kept verbatim when it does not look like a SID tail. SIDs always end in
/// a numeric subauthority; non-SID strings pass through untouched.
fn rid_of_sid(sid: &str) -> String {
    match sid.rsplit('-').next().and_then(|t| t.parse::<u32>().ok()) {
        Some(rid) => rid.to_string(),
        None => sid.to_string(),
    }
}

/// sysinfo reports Windows user ids as SID strings; the kernel's numeric
/// uid table (and the process-widget `uid_to_name` resolution) only matches
/// numbers, so the RID is exposed here. Unix modules keep the sysinfo
/// string as-is.
pub fn read_process_user_id(process: &sysinfo::Process) -> Option<String> {
    process.user_id().map(|uid| rid_of_sid(&uid.to_string()))
}

fn parse_local_users(json: &str) -> Vec<(u32, String)> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return Vec::new();
    };
    let entries = match value {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Object(_) => vec![value],
        _ => return Vec::new(),
    };
    let mut users = Vec::new();
    for entry in entries {
        let Some(name) = entry.get("name").and_then(|n| n.as_str()) else {
            continue;
        };
        let Some(rid) = entry.get("rid").and_then(|r| r.as_u64()) else {
            continue;
        };
        if !name.is_empty() && rid <= u32::MAX as u64 {
            users.push((rid as u32, name.to_string()));
        }
    }
    users
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_users_parse_json_objects_and_arrays() {
        let text = r#"[{"rid":500,"name":"Administrador"},{"rid":1001,"name":"xscriptor"}]"#;
        let users = parse_local_users(text);
        assert_eq!(
            users,
            vec![
                (500, "Administrador".to_string()),
                (1001, "xscriptor".to_string())
            ]
        );

        let single = parse_local_users(r#"{"rid":1001,"name":"xscriptor"}"#);
        assert_eq!(single, vec![(1001, "xscriptor".to_string())]);
    }

    #[test]
    fn local_users_skip_malformed_entries() {
        let text = r#"[{"rid":"500","name":"no-rid"},{"rid":99999999999,"name":"too-big"},{"name":"no-rid-at-all"},"junk"]"#;
        assert!(parse_local_users(text).is_empty());
        assert!(parse_local_users("").is_empty());
        assert!(parse_local_users("not json {").is_empty());
    }

    #[test]
    fn sid_rid_is_the_trailing_subauthority() {
        assert_eq!(
            rid_of_sid("S-1-5-21-755981696-3701129900-2136851656-1000"),
            "1000"
        );
        assert_eq!(rid_of_sid("S-1-5-18"), "18");
        assert_eq!(rid_of_sid("S-1-0-0"), "0");
        assert_eq!(rid_of_sid(""), "");
    }
}
