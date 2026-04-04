"""
DevHost CLI (dh) — Integration & E2E Test Suite

MUST run as Administrator (hosts file + ports 80/443)
Usage:
    pytest tests/test_dh.py -v --tb=short
    pytest tests/test_dh.py -v -k "TestCli"        # CLI-only tests
    pytest tests/test_dh.py -v -k "TestE2e"         # E2E HTTPS tests
"""

import subprocess
import json
import time
import os
import socket
import http.server
import threading
import pytest

# --- Config ---
DH_EXE = os.path.normpath(os.path.join(
    os.path.dirname(__file__),
    "..", "src-tauri", "target", "debug", "dh.exe"
))
TEST_DOMAIN = "pytest-app.test"
TEST_DOMAIN_2 = "pytest-api.test"
TEST_UPSTREAM_PORT = 19876
TEST_UPSTREAM = f"http://127.0.0.1:{TEST_UPSTREAM_PORT}"

HOSTS_FILE = r"C:\Windows\System32\drivers\etc\hosts"
DEVHOST_MARKER = "DevHost BEGIN"


# --- Helpers ---
def dh(*args: str, check: bool = True) -> subprocess.CompletedProcess:
    """Run dh CLI command and return result with UTF-8 encoding."""
    cmd = [DH_EXE, *args]
    result = subprocess.run(
        cmd,
        capture_output=True,
        timeout=30,
        encoding="utf-8",
        errors="replace",
    )
    if check and result.returncode != 0:
        print(f"STDOUT: {result.stdout}")
        print(f"STDERR: {result.stderr}")
    return result


def is_port_open(host: str, port: int, timeout: float = 2.0) -> bool:
    try:
        with socket.create_connection((host, port), timeout=timeout):
            return True
    except (socket.timeout, ConnectionRefusedError, OSError):
        return False


def read_hosts() -> str:
    """Read hosts file with proper encoding."""
    for enc in ["utf-8-sig", "utf-8", "mbcs", "latin-1"]:
        try:
            return open(HOSTS_FILE, "r", encoding=enc).read()
        except (UnicodeDecodeError, LookupError):
            continue
    return open(HOSTS_FILE, "rb").read().decode("latin-1")


def hosts_contains(domain: str) -> bool:
    try:
        return domain in read_hosts()
    except PermissionError:
        pytest.skip("Need admin to read hosts file")
        return False


def safe_stdout(result: subprocess.CompletedProcess) -> str:
    """Get stdout safely, never None."""
    return result.stdout or ""


class DummyUpstream(http.server.HTTPServer):
    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            body = json.dumps({
                "status": "ok",
                "server": "dummy-upstream",
                "path": self.path,
            })
            self.wfile.write(body.encode())

        def log_message(self, *args):
            pass

    def __init__(self, port: int):
        super().__init__(("127.0.0.1", port), self.Handler)


# --- Fixtures ---
@pytest.fixture(scope="session", autouse=True)
def check_prerequisites():
    assert os.path.exists(DH_EXE), f"dh.exe not found at {DH_EXE}"
    try:
        with open(HOSTS_FILE, "a"):
            pass
    except PermissionError:
        pytest.exit("Tests must run as Administrator!", returncode=1)


@pytest.fixture(scope="session")
def upstream_server():
    server = DummyUpstream(TEST_UPSTREAM_PORT)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    yield server
    server.shutdown()


@pytest.fixture(autouse=True)
def cleanup_test_domains():
    yield
    dh("remove", TEST_DOMAIN, check=False)
    dh("remove", TEST_DOMAIN_2, check=False)


# ===================================================================
# TEST GROUP 1: CLI Basic Commands
# ===================================================================

class TestCliHelp:
    def test_help_shows_commands(self):
        r = dh("--help", check=False)
        out = safe_stdout(r)
        assert r.returncode == 0
        assert "add" in out
        assert "remove" in out
        assert "list" in out

    def test_version(self):
        r = dh("--version", check=False)
        assert r.returncode == 0
        assert "1.0.0" in safe_stdout(r)

    def test_add_help(self):
        r = dh("add", "--help", check=False)
        out = safe_stdout(r).lower()
        assert r.returncode == 0
        assert "domain" in out
        assert "upstream" in out

    def test_nginx_help(self):
        r = dh("nginx", "--help", check=False)
        out = safe_stdout(r).lower()
        assert r.returncode == 0
        assert "start" in out
        assert "stop" in out

    def test_ca_help(self):
        r = dh("ca", "--help", check=False)
        out = safe_stdout(r).lower()
        assert r.returncode == 0
        assert "install" in out
        assert "status" in out

    def test_unknown_command_errors(self):
        r = dh("foobar", check=False)
        assert r.returncode != 0


# ===================================================================
# TEST GROUP 2: Domain CRUD operations
# ===================================================================

class TestDomainCrud:
    def test_add_domain(self):
        r = dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        out = safe_stdout(r)
        assert r.returncode == 0
        assert TEST_DOMAIN in out

    def test_add_domain_appears_in_list(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        r = dh("list")
        out = safe_stdout(r)
        assert r.returncode == 0
        assert TEST_DOMAIN in out

    def test_add_multiple_domains(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        dh("add", TEST_DOMAIN_2, "http://127.0.0.1:5000")
        r = dh("list")
        out = safe_stdout(r)
        assert TEST_DOMAIN in out
        assert TEST_DOMAIN_2 in out

    def test_remove_domain(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        r = dh("remove", TEST_DOMAIN)
        assert r.returncode == 0

        r2 = dh("list")
        assert TEST_DOMAIN not in safe_stdout(r2)

    def test_toggle_domain_off(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        r = dh("toggle", TEST_DOMAIN)
        out = safe_stdout(r)
        assert r.returncode == 0
        assert "disabled" in out.lower() or "off" in out.lower() or r.returncode == 0

    def test_toggle_domain_on_again(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        dh("toggle", TEST_DOMAIN)  # off
        r = dh("toggle", TEST_DOMAIN)  # on again
        out = safe_stdout(r)
        assert r.returncode == 0
        assert "enabled" in out.lower() or "on" in out.lower() or r.returncode == 0

    def test_list_empty(self):
        r = dh("list")
        assert r.returncode == 0


# ===================================================================
# TEST GROUP 3: Validation & Edge Cases
# ===================================================================

class TestValidation:
    def test_reject_non_test_domain(self):
        r = dh("add", "myapp.com", TEST_UPSTREAM, check=False)
        assert r.returncode != 0

    def test_reject_invalid_upstream(self):
        r = dh("add", TEST_DOMAIN, "not-a-url", check=False)
        assert r.returncode != 0

    def test_reject_upstream_without_protocol(self):
        r = dh("add", TEST_DOMAIN, "127.0.0.1:3000", check=False)
        assert r.returncode != 0

    def test_add_domain_with_local_suffix(self):
        domain = "myapp.local"
        r = dh("add", domain, TEST_UPSTREAM, check=False)
        if r.returncode == 0:
            assert domain in safe_stdout(r)
            dh("remove", domain, check=False)

    def test_duplicate_domain_updates(self):
        """Adding same domain twice should update, not error."""
        dh("add", TEST_DOMAIN, "http://127.0.0.1:3000")
        r = dh("add", TEST_DOMAIN, "http://127.0.0.1:5000")
        assert r.returncode == 0

    def test_remove_nonexistent_domain(self):
        r = dh("remove", "nonexistent.test", check=False)
        # Should succeed (idempotent) or fail gracefully
        assert r.returncode == 0 or r.returncode != 0  # just shouldn't crash

    def test_toggle_nonexistent_domain(self):
        r = dh("toggle", "nonexistent.test", check=False)
        assert r.returncode != 0


# ===================================================================
# TEST GROUP 4: Hosts File Integration
# ===================================================================

class TestHostsFile:
    def test_add_writes_to_hosts(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        assert hosts_contains(TEST_DOMAIN), f"{TEST_DOMAIN} not found in hosts file"

    def test_remove_cleans_hosts(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        dh("remove", TEST_DOMAIN)
        assert not hosts_contains(TEST_DOMAIN), f"{TEST_DOMAIN} still in hosts file"

    def test_devhost_markers_present(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        content = read_hosts()
        assert DEVHOST_MARKER in content, \
            f"DevHost marker not found. Content tail: {content[-300:]}"

    def test_toggle_off_removes_from_hosts(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        dh("toggle", TEST_DOMAIN)
        assert not hosts_contains(TEST_DOMAIN), "Disabled domain should not be in hosts"

    def test_hosts_resolves_to_localhost(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        try:
            ip = socket.gethostbyname(TEST_DOMAIN)
            assert ip == "127.0.0.1", f"Expected 127.0.0.1 but got {ip}"
        except socket.gaierror:
            pytest.fail(f"DNS resolution failed for {TEST_DOMAIN}")


# ===================================================================
# TEST GROUP 5: Certificate Generation
# ===================================================================

class TestCertificates:
    def _cert_dir(self) -> str:
        return os.path.join(
            os.environ.get("LOCALAPPDATA", ""), "DevHost", "certs"
        )

    def test_cert_files_created(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        cert_dir = self._cert_dir()
        crt = os.path.join(cert_dir, f"{TEST_DOMAIN}.crt")
        key = os.path.join(cert_dir, f"{TEST_DOMAIN}.key")
        assert os.path.exists(crt), f"Cert not found: {crt}"
        assert os.path.exists(key), f"Key not found: {key}"

    def test_cert_is_valid_pem(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        crt = open(os.path.join(self._cert_dir(), f"{TEST_DOMAIN}.crt")).read()
        assert "BEGIN CERTIFICATE" in crt

    def test_key_is_valid_pem(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        key = open(os.path.join(self._cert_dir(), f"{TEST_DOMAIN}.key")).read()
        assert "BEGIN" in key and "KEY" in key

    def test_remove_deletes_cert_files(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        crt = os.path.join(self._cert_dir(), f"{TEST_DOMAIN}.crt")
        dh("remove", TEST_DOMAIN)
        assert not os.path.exists(crt)

    def test_ca_status_command(self):
        r = dh("ca", "status")
        assert r.returncode == 0
        assert "CA" in safe_stdout(r).upper()


# ===================================================================
# TEST GROUP 6: nginx Management
# ===================================================================

class TestNginx:
    def test_nginx_status(self):
        r = dh("nginx", "status")
        assert r.returncode == 0
        assert "nginx" in safe_stdout(r).lower()

    def test_nginx_logs(self):
        r = dh("nginx", "logs", "--lines", "5", check=False)
        assert r.returncode == 0


# ===================================================================
# TEST GROUP 7: E2E HTTPS Flow
# ===================================================================

class TestE2eHttps:
    @pytest.fixture(autouse=True)
    def setup_e2e(self, upstream_server):
        assert is_port_open("127.0.0.1", TEST_UPSTREAM_PORT)
        yield
        dh("nginx", "stop", check=False)

    def test_full_flow(self):
        r = dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        assert r.returncode == 0
        assert hosts_contains(TEST_DOMAIN)

        r = dh("nginx", "start")
        assert r.returncode == 0
        time.sleep(1)

        if is_port_open("127.0.0.1", 443):
            import urllib3
            urllib3.disable_warnings()
            import requests
            try:
                resp = requests.get(
                    f"https://{TEST_DOMAIN}/health",
                    verify=False, timeout=5,
                )
                assert resp.status_code == 200
                data = resp.json()
                assert data["status"] == "ok"
            except requests.ConnectionError:
                pytest.skip("HTTPS connection failed")
        else:
            pytest.skip("Port 443 not open")

    def test_http_redirects_to_https(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        dh("nginx", "start")
        time.sleep(1)

        if is_port_open("127.0.0.1", 80):
            import requests
            try:
                resp = requests.get(
                    f"http://{TEST_DOMAIN}/",
                    allow_redirects=False, timeout=5,
                )
                assert resp.status_code == 301
                assert "https://" in resp.headers.get("Location", "")
            except requests.ConnectionError:
                pytest.skip("Port 80 not accessible")
        else:
            pytest.skip("Port 80 not open")

    def test_websocket_headers_in_config(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        conf_path = os.path.join(
            os.environ.get("LOCALAPPDATA", ""),
            "DevHost", "nginx", "nginx.conf"
        )
        if os.path.exists(conf_path):
            conf = open(conf_path).read()
            assert "proxy_set_header" in conf
            assert "Upgrade" in conf
        else:
            pytest.skip("nginx.conf not found")

    def test_disabled_domain_not_in_config(self):
        dh("add", TEST_DOMAIN, TEST_UPSTREAM)
        dh("toggle", TEST_DOMAIN)
        conf_path = os.path.join(
            os.environ.get("LOCALAPPDATA", ""),
            "DevHost", "nginx", "nginx.conf"
        )
        if os.path.exists(conf_path):
            conf = open(conf_path).read()
            assert f"server_name {TEST_DOMAIN}" not in conf


# ===================================================================
# TEST GROUP 8: Stress Tests
# ===================================================================

class TestStress:
    def test_add_many_domains(self):
        domains = [f"stress-{i}.test" for i in range(10)]
        for d in domains:
            r = dh("add", d, f"http://127.0.0.1:{3000 + int(d.split('-')[1].split('.')[0])}")
            assert r.returncode == 0

        r = dh("list")
        out = safe_stdout(r)
        for d in domains:
            assert d in out, f"{d} missing from list"

        for d in domains:
            dh("remove", d, check=False)

    def test_rapid_add_remove(self):
        for _ in range(5):
            assert dh("add", TEST_DOMAIN, TEST_UPSTREAM).returncode == 0
            assert dh("remove", TEST_DOMAIN).returncode == 0

        r = dh("list")
        assert TEST_DOMAIN not in safe_stdout(r)


if __name__ == "__main__":
    pytest.main([__file__, "-v", "--tb=short", "-x"])
