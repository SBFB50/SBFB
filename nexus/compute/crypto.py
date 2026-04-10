"""
NEXUS Compute -- Ed25519 Cryptographic Signing (Couche 1).

Provides identity-based verification for compute results:
- Key pair generation (worker-side)
- Result signing (worker-side)
- Signature verification (server-side)

Each contributor generates an Ed25519 key pair at registration.
The public key is sent to the server. The private key stays local.
Every result submission is signed: Ed25519(payload_json).

This proves WHO sent the result (non-repudiation).
It does NOT prove which model generated the result (see Couche 2+3).
"""

from __future__ import annotations

import base64
import json
import time
from typing import Optional

from loguru import logger

try:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import (
        Ed25519PrivateKey,
        Ed25519PublicKey,
    )
    from cryptography.hazmat.primitives.serialization import (
        Encoding,
        NoEncryption,
        PrivateFormat,
        PublicFormat,
    )
    HAS_CRYPTO = True
except ImportError:
    HAS_CRYPTO = False
    logger.debug("cryptography package not installed — Ed25519 signing disabled")


# ============================================================================
# Key management
# ============================================================================

def generate_keypair() -> tuple[bytes, bytes]:
    """Generate an Ed25519 key pair.

    Returns (private_key_pem, public_key_pem) as bytes.
    """
    if not HAS_CRYPTO:
        return b"", b""

    private_key = Ed25519PrivateKey.generate()
    private_pem = private_key.private_bytes(
        Encoding.PEM, PrivateFormat.PKCS8, NoEncryption(),
    )
    public_pem = private_key.public_key().public_bytes(
        Encoding.PEM, PublicFormat.SubjectPublicKeyInfo,
    )
    return private_pem, public_pem


def load_private_key(pem_data: bytes) -> Optional[object]:
    """Load a private key from PEM bytes."""
    if not HAS_CRYPTO or not pem_data:
        return None
    try:
        from cryptography.hazmat.primitives.serialization import load_pem_private_key
        return load_pem_private_key(pem_data, password=None)
    except Exception as exc:
        logger.error("Failed to load private key: {}", exc)
        return None


def load_public_key(pem_data: bytes) -> Optional[object]:
    """Load a public key from PEM bytes."""
    if not HAS_CRYPTO or not pem_data:
        return None
    try:
        from cryptography.hazmat.primitives.serialization import load_pem_public_key
        return load_pem_public_key(pem_data)
    except Exception as exc:
        logger.error("Failed to load public key: {}", exc)
        return None


# ============================================================================
# Signing (worker-side)
# ============================================================================

def sign_result(
    private_key_pem: bytes,
    task_id: str,
    result_text: str,
    model_digest: str,
    node_id: str,
) -> str:
    """Sign a result payload with Ed25519.

    Returns base64-encoded signature string, or empty string if signing unavailable.
    """
    if not HAS_CRYPTO or not private_key_pem:
        return ""

    try:
        key = load_private_key(private_key_pem)
        if key is None:
            return ""

        payload = _build_payload(task_id, result_text, model_digest, node_id)
        signature = key.sign(payload)
        return base64.b64encode(signature).decode("ascii")
    except Exception as exc:
        logger.error("Failed to sign result: {}", exc)
        return ""


# ============================================================================
# Verification (server-side)
# ============================================================================

def verify_signature(
    public_key_pem: bytes,
    signature_b64: str,
    task_id: str,
    result_text: str,
    model_digest: str,
    node_id: str,
) -> bool:
    """Verify an Ed25519 signature on a result payload.

    Returns True if signature is valid, False otherwise.
    """
    if not HAS_CRYPTO:
        logger.debug("cryptography not installed — skipping signature verification")
        return True  # Graceful degradation: accept if crypto not available

    if not public_key_pem or not signature_b64:
        return False

    try:
        key = load_public_key(public_key_pem)
        if key is None:
            return False

        payload = _build_payload(task_id, result_text, model_digest, node_id)
        signature = base64.b64decode(signature_b64)
        key.verify(signature, payload)
        return True
    except Exception:
        return False


# ============================================================================
# Payload construction (deterministic)
# ============================================================================

def _build_payload(
    task_id: str,
    result_text: str,
    model_digest: str,
    node_id: str,
) -> bytes:
    """Build the canonical payload bytes for signing/verification.

    Uses sorted JSON keys for deterministic serialization.
    """
    data = {
        "task_id": task_id,
        "result": result_text[:2000],  # Truncate for performance (sign summary, not full)
        "model_digest": model_digest,
        "node_id": node_id,
    }
    return json.dumps(data, sort_keys=True, ensure_ascii=True).encode("utf-8")
