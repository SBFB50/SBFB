# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 14 Phase A — tests for forge URL detection and helpers."""

from __future__ import annotations

from nexus_coordinator.forge import (
    ForgeType,
    detect_forge,
    normalize_clone_url,
    raw_file_url,
)


class TestDetectForge:
    def test_github(self) -> None:
        info = detect_forge("https://github.com/alice/my-app")
        assert info.forge == ForgeType.GITHUB
        assert info.owner == "alice"
        assert info.repo == "my-app"

    def test_gitlab(self) -> None:
        info = detect_forge("https://gitlab.com/bob/cool-project")
        assert info.forge == ForgeType.GITLAB
        assert info.owner == "bob"
        assert info.repo == "cool-project"

    def test_codeberg(self) -> None:
        info = detect_forge("https://codeberg.org/carol/sbfb-chat")
        assert info.forge == ForgeType.CODEBERG
        assert info.owner == "carol"
        assert info.repo == "sbfb-chat"

    def test_gitea_self_hosted(self) -> None:
        info = detect_forge("https://git.example.com/dave/my-thing")
        assert info.forge == ForgeType.GITEA
        assert info.owner == "dave"
        assert info.repo == "my-thing"
        assert info.host == "git.example.com"

    def test_unknown_protocol(self) -> None:
        info = detect_forge("ftp://some-server/repo")
        assert info.forge == ForgeType.UNKNOWN

    def test_github_with_git_suffix(self) -> None:
        info = detect_forge("https://github.com/alice/my-app.git")
        assert info.forge == ForgeType.GITHUB
        assert info.repo == "my-app"


class TestRawFileUrl:
    def test_github(self) -> None:
        url = raw_file_url("https://github.com/alice/app", "SBFB.json")
        assert url == "https://raw.githubusercontent.com/alice/app/HEAD/SBFB.json"

    def test_gitlab(self) -> None:
        url = raw_file_url("https://gitlab.com/bob/proj", "SBFB.json", ref="main")
        assert url == "https://gitlab.com/bob/proj/-/raw/main/SBFB.json"

    def test_codeberg(self) -> None:
        url = raw_file_url("https://codeberg.org/carol/chat", "SBFB.json")
        assert url == "https://codeberg.org/carol/chat/raw/branch/HEAD/SBFB.json"

    def test_gitea(self) -> None:
        url = raw_file_url("https://git.example.com/dave/app", "index.html")
        assert url == "https://git.example.com/dave/app/raw/branch/HEAD/index.html"


class TestNormalizeCloneUrl:
    def test_strips_trailing_slash(self) -> None:
        assert normalize_clone_url("https://github.com/a/b/") == "https://github.com/a/b"

    def test_strips_fragment(self) -> None:
        assert normalize_clone_url("https://github.com/a/b#readme") == "https://github.com/a/b"

    def test_strips_query(self) -> None:
        assert normalize_clone_url("https://github.com/a/b?tab=about") == "https://github.com/a/b"

    def test_strips_git_suffix(self) -> None:
        assert normalize_clone_url("https://github.com/a/b.git") == "https://github.com/a/b"

    def test_whitespace_trimmed(self) -> None:
        assert normalize_clone_url("  https://github.com/a/b  ") == "https://github.com/a/b"
