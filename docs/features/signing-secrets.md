# Signing Secrets

## Overview

Pigeon generates signing secrets for each endpoint. Secrets are used to compute HMAC-SHA256 signatures on webhook payloads, allowing consumers to verify that deliveries are authentic.

Secret format: `whsec_` followed by 64 hex characters (32 random bytes).

## Lifecycle

1. **Create endpoint** — Pigeon generates a secret automatically. The full secret is returned once in the response. Copy it — you won't see it again.
2. **Normal operation** — secrets are shown as masked (`whsec_...abc123`) in all API responses and the UI.
3. **Rotate** — adds a new secret, keeps the old one. Max 2 active secrets.
4. **Revoke** — removes the old secret after your consumer has updated.

## Rotation (zero downtime)

```
POST /api/v1/applications/{app_id}/endpoints/{id}/rotate
```

Returns the new secret in the response (shown once).

During the transition window, Pigeon signs every delivery with **all** active secrets:

```
X-Pigeon-Signature: sha256=<new_sig>,sha256=<old_sig>
```

Your consumer can verify against either signature. Update your consumer to use the new secret, then revoke the old one:

```
DELETE /api/v1/applications/{app_id}/endpoints/{id}/secrets/1
```

Index `0` is the newest secret, `1` is the oldest.

## Verification (consumer side)

```python
import hmac
import hashlib

def verify(payload: bytes, secret: str, signature_header: str) -> bool:
    expected = hmac.new(secret.encode(), payload, hashlib.sha256).hexdigest()
    for sig in signature_header.split(","):
        if sig.strip().removeprefix("sha256=") == expected:
            return True
    return False
```
