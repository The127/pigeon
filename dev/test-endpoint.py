#!/usr/bin/env python3
"""
Test webhook endpoint for Pigeon development.
Logs incoming webhooks, verifies HMAC-SHA256 signatures, and pretty-prints payloads.

Usage:
    just dev-endpoint
    just dev-endpoint 9999
    just dev-endpoint 9999 my-secret

Default: port 8888, secret "test-secret"
"""

import hashlib
import hmac
import json
import sys
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler

PORT = int(sys.argv[1]) if len(sys.argv) > 1 else 8888
SECRET = sys.argv[2] if len(sys.argv) > 2 else "test-secret"

RESET = "\033[0m"
BOLD = "\033[1m"
DIM = "\033[2m"
GREEN = "\033[32m"
RED = "\033[31m"
YELLOW = "\033[33m"
CYAN = "\033[36m"


def verify_signature(body: bytes, signature_header: str | None, secret: str) -> tuple[bool, str]:
    if not signature_header:
        return False, "missing"

    if not signature_header.startswith("sha256="):
        return False, f"bad format: {signature_header}"

    expected = hmac.new(secret.encode(), body, hashlib.sha256).hexdigest()
    received = signature_header[7:]

    if hmac.compare_digest(expected, received):
        return True, "valid"
    else:
        return False, f"mismatch (got {received[:16]}..., expected {expected[:16]}...)"


class WebhookHandler(BaseHTTPRequestHandler):
    def do_POST(self):
        content_length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_length)

        now = datetime.now(timezone.utc).strftime("%H:%M:%S")
        sig_header = self.headers.get("X-Pigeon-Signature")
        sig_valid, sig_detail = verify_signature(body, sig_header, SECRET)
        request_id = self.headers.get("x-request-id", "—")

        # Parse body
        try:
            payload = json.loads(body)
            payload_str = json.dumps(payload, indent=2)
        except json.JSONDecodeError:
            payload_str = body.decode("utf-8", errors="replace")

        # Print
        sig_color = GREEN if sig_valid else RED
        sig_icon = "✓" if sig_valid else "✗"

        print(f"\n{DIM}{'─' * 60}{RESET}")
        print(f"{BOLD}{CYAN}▶ POST {self.path}{RESET}  {DIM}{now}{RESET}")
        print(f"  {DIM}Request-ID:{RESET}  {request_id}")
        print(f"  {DIM}Signature:{RESET}   {sig_color}{sig_icon} {sig_detail}{RESET}")
        print(f"  {DIM}Content-Type:{RESET} {self.headers.get('Content-Type', '—')}")

        # Print select Pigeon headers
        for header in sorted(self.headers.keys()):
            lower = header.lower()
            if lower.startswith("x-pigeon-") and lower != "x-pigeon-signature":
                print(f"  {DIM}{header}:{RESET} {self.headers[header]}")

        print(f"\n{payload_str}\n")

        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps({"status": "ok"}).encode())

    def log_message(self, format, *args):
        pass  # Suppress default access log


def main():
    server = HTTPServer(("0.0.0.0", PORT), WebhookHandler)
    print(f"{BOLD}Pigeon test endpoint{RESET}")
    print(f"  {DIM}Listening:{RESET}  http://localhost:{PORT}")
    print(f"  {DIM}Secret:{RESET}    {SECRET}")
    print(f"\n  {DIM}Use this URL when creating endpoints:{RESET}")
    print(f"  {CYAN}http://host.docker.internal:{PORT}/webhook{RESET}")
    print(f"  {DIM}or{RESET}")
    print(f"  {CYAN}http://172.17.0.1:{PORT}/webhook{RESET}")
    print(f"\n{DIM}Waiting for webhooks...{RESET}\n")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print(f"\n{DIM}Shutting down.{RESET}")
        server.server_close()


if __name__ == "__main__":
    main()
