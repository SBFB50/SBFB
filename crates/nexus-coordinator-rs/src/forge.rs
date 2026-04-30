// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-forge URL detection and helpers for verified deploy
//! (Sprint 42 Phase B, port of forge.py S14).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeType {
    GitHub,
    GitLab,
    Codeberg,
    Gitea,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForgeInfo {
    pub forge: ForgeType,
    pub owner: String,
    pub repo: String,
    pub host: String,
}

pub fn normalize_clone_url(repo_url: &str) -> String {
    let mut url = repo_url.trim().to_string();
    if let Some(pos) = url.find('#') {
        url.truncate(pos);
    }
    if let Some(pos) = url.find('?') {
        url.truncate(pos);
    }
    let url = url.trim_end_matches('/');
    let url = url.strip_suffix(".git").unwrap_or(url);
    url.to_string()
}

pub fn detect_forge(repo_url: &str) -> ForgeInfo {
    let url = normalize_clone_url(repo_url);
    let unknown = ForgeInfo {
        forge: ForgeType::Unknown,
        owner: String::new(),
        repo: String::new(),
        host: String::new(),
    };

    let stripped = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or_default();

    if stripped.is_empty() {
        return unknown;
    }

    let parts: Vec<&str> = stripped.splitn(4, '/').collect();
    if parts.len() < 3 {
        return unknown;
    }

    let host = parts[0];
    let owner = parts[1].to_string();
    let repo = parts[2].to_string();

    let forge = match host {
        "github.com" => ForgeType::GitHub,
        "gitlab.com" => ForgeType::GitLab,
        "codeberg.org" => ForgeType::Codeberg,
        _ => ForgeType::Gitea,
    };

    ForgeInfo {
        forge,
        owner,
        repo,
        host: host.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_git_suffix() {
        assert_eq!(
            normalize_clone_url("https://github.com/user/repo.git"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn normalize_strips_trailing_slash() {
        assert_eq!(
            normalize_clone_url("https://github.com/user/repo/"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn normalize_strips_fragment_and_query() {
        assert_eq!(
            normalize_clone_url("https://github.com/user/repo?tab=code#readme"),
            "https://github.com/user/repo"
        );
    }

    #[test]
    fn detect_github() {
        let info = detect_forge("https://github.com/user/repo.git");
        assert_eq!(info.forge, ForgeType::GitHub);
        assert_eq!(info.owner, "user");
        assert_eq!(info.repo, "repo");
    }

    #[test]
    fn detect_gitlab() {
        let info = detect_forge("https://gitlab.com/org/project");
        assert_eq!(info.forge, ForgeType::GitLab);
        assert_eq!(info.owner, "org");
        assert_eq!(info.repo, "project");
    }

    #[test]
    fn detect_codeberg() {
        let info = detect_forge("https://codeberg.org/user/repo");
        assert_eq!(info.forge, ForgeType::Codeberg);
    }

    #[test]
    fn detect_self_hosted_gitea() {
        let info = detect_forge("https://git.example.com/user/repo");
        assert_eq!(info.forge, ForgeType::Gitea);
        assert_eq!(info.host, "git.example.com");
    }

    #[test]
    fn detect_unknown_no_path() {
        let info = detect_forge("https://example.com");
        assert_eq!(info.forge, ForgeType::Unknown);
    }
}
