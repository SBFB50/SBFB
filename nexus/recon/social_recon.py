"""
NEXUS -- Social profile reconnaissance.

Checks whether a username exists on major social platforms via
async HTTP HEAD requests.  Rate-limited to max 5 concurrent requests
to avoid getting blocked.
"""

from __future__ import annotations

import asyncio
from typing import Any, Dict, List

import httpx
from loguru import logger


# Platforms to check.  The ``{}`` placeholder is replaced by the username.
PLATFORMS: Dict[str, str] = {
    "twitter": "https://x.com/{}",
    "instagram": "https://www.instagram.com/{}/",
    "github": "https://github.com/{}",
    "linkedin": "https://www.linkedin.com/in/{}/",
    "facebook": "https://www.facebook.com/{}",
    "tiktok": "https://www.tiktok.com/@{}",
    "reddit": "https://www.reddit.com/user/{}/",
    "pinterest": "https://www.pinterest.com/{}/",
    "youtube": "https://www.youtube.com/@{}",
    "telegram": "https://t.me/{}",
}

# Max concurrent platform checks
_MAX_CONCURRENT = 5


class SocialRecon:
    """Username/profile existence checker across major social platforms.

    Usage::

        recon = SocialRecon()
        profiles = await recon.search_username("johndoe42")
    """

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def search_username(self, username: str) -> List[Dict[str, Any]]:
        """Check *username* across all configured platforms.

        For each platform a HEAD request is sent.  A 200 (or redirect to
        a profile page) is treated as "exists", 404 as "not found".
        Other status codes are recorded but ``exists`` is set to False.

        Returns:
            List of dicts: platform, url, exists (bool), status_code.
        """
        username = username.strip()
        if not username:
            return []

        logger.info("SocialRecon: searching username '{}'", username)

        semaphore = asyncio.Semaphore(_MAX_CONCURRENT)
        results: List[Dict[str, Any]] = []

        async def _check(platform: str, url_template: str) -> Dict[str, Any]:
            url = url_template.format(username)
            async with semaphore:
                try:
                    async with httpx.AsyncClient(
                        timeout=15.0,
                        follow_redirects=True,
                        headers={
                            "User-Agent": (
                                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                                "AppleWebKit/537.36 (KHTML, like Gecko) "
                                "Chrome/125.0.0.0 Safari/537.36"
                            ),
                        },
                    ) as client:
                        resp = await client.head(url)
                        status = resp.status_code

                        # Some platforms return 200 even for missing profiles
                        # but most return 404 for unknown usernames
                        exists = status == 200

                        return {
                            "platform": platform,
                            "url": url,
                            "exists": exists,
                            "status_code": status,
                        }
                except httpx.TimeoutException:
                    logger.debug(
                        "SocialRecon: timeout checking {} for '{}'",
                        platform, username,
                    )
                    return {
                        "platform": platform,
                        "url": url,
                        "exists": False,
                        "status_code": 0,
                    }
                except Exception:
                    logger.debug(
                        "SocialRecon: error checking {} for '{}'",
                        platform, username,
                    )
                    return {
                        "platform": platform,
                        "url": url,
                        "exists": False,
                        "status_code": 0,
                    }

        tasks = [
            _check(platform, url_tpl)
            for platform, url_tpl in PLATFORMS.items()
        ]
        results = await asyncio.gather(*tasks)

        found_count = sum(1 for r in results if r["exists"])
        logger.info(
            "SocialRecon: username '{}' -- {}/{} platforms returned exists",
            username, found_count, len(PLATFORMS),
        )
        return list(results)

    async def search_email_username(self, email: str) -> List[Dict[str, Any]]:
        """Extract the local part of *email* and search it as a username.

        Example: ``john.doe@gmail.com`` -> searches for ``john.doe``.
        """
        email = email.strip().lower()
        if "@" not in email:
            return []
        username = email.split("@")[0]
        return await self.search_username(username)
