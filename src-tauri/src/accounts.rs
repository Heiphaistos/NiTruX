use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct UserAccount {
    pub username: String,
    pub uid: u32,
    pub home: String,
    pub shell: String,
}

/// Parses one `/etc/passwd` line (colon-separated: username, password
/// placeholder, uid, gid, gecos, home, shell). Returns `None` for a
/// malformed line (wrong field count) or a non-numeric uid, rather than
/// panicking -- a hand-edited or unusual passwd file should degrade to
/// "skip this line", never crash the whole listing.
pub fn parse_passwd_line(line: &str) -> Option<UserAccount> {
    let fields: Vec<&str> = line.split(':').collect();
    if fields.len() < 7 {
        return None;
    }
    let uid: u32 = fields[2].parse().ok()?;
    Some(UserAccount {
        username: fields[0].to_string(),
        uid,
        home: fields[5].to_string(),
        shell: fields[6].to_string(),
    })
}

/// True for a "real" user account, as opposed to a system/service account
/// -- the standard Linux convention (also used by tools like `useradd`'s
/// own defaults): UIDs below 1000 are system accounts (root, daemon
/// users, ...), and UIDs at/above 60000 are typically nobody/nfsnobody-
/// style reserved ranges, not real people either.
pub fn is_real_user_account(account: &UserAccount) -> bool {
    account.uid >= 1000 && account.uid < 60000
}

#[tauri::command]
pub fn get_user_accounts() -> Vec<UserAccount> {
    std::fs::read_to_string("/etc/passwd")
        .ok()
        .map(|content| {
            content
                .lines()
                .filter_map(parse_passwd_line)
                .filter(is_real_user_account)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_passwd_line() {
        let line = "dev:x:1000:1000:dev,,,:/home/dev:/bin/bash";
        let account = parse_passwd_line(line).expect("should parse");
        assert_eq!(account.username, "dev");
        assert_eq!(account.uid, 1000);
        assert_eq!(account.home, "/home/dev");
        assert_eq!(account.shell, "/bin/bash");
    }

    #[test]
    fn returns_none_for_a_malformed_line() {
        assert!(parse_passwd_line("not:enough:fields").is_none());
    }

    #[test]
    fn is_real_user_account_excludes_system_accounts() {
        let root = UserAccount { username: "root".to_string(), uid: 0, home: "/root".to_string(), shell: "/bin/bash".to_string() };
        let daemon = UserAccount { username: "daemon".to_string(), uid: 999, home: "/nonexistent".to_string(), shell: "/usr/sbin/nologin".to_string() };
        assert!(!is_real_user_account(&root));
        assert!(!is_real_user_account(&daemon));
    }

    #[test]
    fn is_real_user_account_includes_a_normal_user() {
        let dev = UserAccount { username: "dev".to_string(), uid: 1000, home: "/home/dev".to_string(), shell: "/bin/bash".to_string() };
        assert!(is_real_user_account(&dev));
    }
}
