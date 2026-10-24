#!/usr/bin/env python3
"""A throwaway staging index on localhost: an npm registry and a PyPI simple index.

The release's packages are published to it and installed from it exactly as a
user installs them (`npm i`, `npx`, `pip install`, `uvx`), before anything
reaches a real registry (SR-PACKAGES decision 11; W-18). It is the standard
library only, so it adds no dependency to vet (L2), and it listens on
127.0.0.1 alone: nothing here can reach a remote registry.

    staging-index.py DIR [--port-file F]   serve; DIR holds what is published to it

npm  (the registry API `npm publish` and `npm install` speak)
    PUT  /<name>                  `npm publish`; a version published twice is refused (409), as npm does
    GET  /<name>                  the packument; `latest` is the newest version published
    GET  /<name>/-/<file>.tgz     the tarball
PyPI (the legacy upload API and PEP 503's simple index)
    POST /legacy/                 `uv publish` / `twine upload`; the sha256 must match
    GET  /simple/ , /simple/<project>/ , /files/<project>/<file>

Anything else is 404: the index never forwards a request elsewhere. It binds a
free port and writes `http://127.0.0.1:<port>` to --port-file once it listens.
Python 3.9 syntax (work/MACHINE.md).
"""
from __future__ import annotations

import argparse
import base64
import email.parser
import email.policy
import hashlib
import html
import json
import pathlib
import re
import socketserver
import sys
import threading
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

LOCK = threading.Lock()
NPM_NAME = re.compile(r"^(@[a-z0-9][a-z0-9._-]*/)?[a-z0-9][a-z0-9._-]*$")


def normalize(project: str) -> str:  # PEP 503
    return re.sub(r"[-_.]+", "-", project).lower()


def version_key(v: str) -> tuple:
    core, _, pre = v.partition("-")
    nums = tuple(int(x) if x.isdigit() else 0 for x in core.split("."))
    return nums + ((1,) if not pre else (0, pre))  # a pre-release sorts before its release


class Handler(BaseHTTPRequestHandler):
    root: pathlib.Path
    server_version = "codelaw-staging"

    def log_message(self, fmt: str, *args: object) -> None:
        sys.stderr.write("staging-index: %s\n" % (fmt % args))

    def send(self, code: int, body: bytes = b"", ctype: str = "application/json") -> None:
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)

    def error(self, code: int, msg: str) -> None:
        self.send(code, json.dumps({"error": msg}).encode())

    def base(self) -> str:
        return "http://%s:%d" % self.server.server_address[:2]

    def body(self) -> bytes:
        return self.rfile.read(int(self.headers.get("Content-Length") or 0))

    # ── npm ──
    def npm_dir(self, name: str) -> pathlib.Path:
        return self.root / "npm" / name.replace("/", "%2f")

    def npm_name(self, segment: str) -> str:
        return urllib.parse.unquote(segment)  # npm sends `@scope%2fname`

    def do_PUT(self) -> None:
        name = self.npm_name(self.path.split("?")[0].lstrip("/"))
        if not NPM_NAME.match(name):
            return self.error(400, "not a package name: %s" % name)
        doc = json.loads(self.body())
        versions, files = doc.get("versions") or {}, doc.get("_attachments") or {}
        if doc.get("name") != name or len(versions) != 1 or len(files) != 1:
            return self.error(400, "one version and one tarball of %s, please" % name)
        (v, manifest), (fname, att) = next(iter(versions.items())), next(iter(files.items()))
        data = base64.b64decode(att["data"])
        dist = manifest.get("dist") or {}
        if dist.get("shasum") and hashlib.sha1(data).hexdigest() != dist["shasum"]:
            return self.error(400, "%s: the tarball does not match its shasum" % fname)
        if dist.get("integrity", "").startswith("sha512-"):
            if "sha512-" + base64.b64encode(hashlib.sha512(data).digest()).decode() != dist["integrity"]:
                return self.error(400, "%s: the tarball does not match its integrity" % fname)
        d = self.npm_dir(name)
        tgz = pathlib.PurePosixPath(fname).name
        with LOCK:
            meta_file = d / "packument.json"
            meta = json.loads(meta_file.read_text("utf-8")) if meta_file.exists() else {"name": name, "versions": {}, "dist-tags": {}}
            if v in meta["versions"]:
                return self.error(409, "%s@%s is already published" % (name, v))
            d.mkdir(parents=True, exist_ok=True)
            (d / tgz).write_bytes(data)
            dist["tarball"] = "%s/%s/-/%s" % (self.base(), name, tgz)
            manifest["dist"] = dist
            meta["versions"][v] = manifest
            meta["dist-tags"].update(doc.get("dist-tags") or {})
            meta["dist-tags"]["latest"] = max(meta["versions"], key=version_key)
            meta_file.write_text(json.dumps(meta), "utf-8")
        self.send(201, json.dumps({"ok": True, "id": name}).encode())

    def npm_get(self, path: str) -> None:
        name, sep, tgz = path.partition("/-/")
        name = self.npm_name(name)
        d = self.npm_dir(name)
        if sep:
            f = d / pathlib.PurePosixPath(tgz).name
            return self.send(200, f.read_bytes(), "application/octet-stream") if f.is_file() else self.error(404, "no tarball %s" % tgz)
        meta_file = d / "packument.json"
        if not NPM_NAME.match(name) or not meta_file.is_file():
            return self.error(404, "%s is not on the staging index" % name)
        meta = json.loads(meta_file.read_text("utf-8"))
        for m in meta["versions"].values():  # the address it is served at now
            m["dist"]["tarball"] = "%s/%s/-/%s" % (self.base(), name, m["dist"]["tarball"].rsplit("/-/", 1)[1])
        self.send(200, json.dumps(meta).encode())

    # ── PyPI ──
    def do_POST(self) -> None:
        if self.path.split("?")[0].rstrip("/") != "/legacy":
            return self.error(404, "uploads go to /legacy/")
        ctype = self.headers.get("Content-Type", "")
        msg = email.parser.BytesParser(policy=email.policy.HTTP).parsebytes(
            b"Content-Type: " + ctype.encode() + b"\r\n\r\n" + self.body())
        form, content, fname = {}, None, None
        for part in msg.iter_parts():
            key = part.get_param("name", header="content-disposition")
            if key == "content":
                content, fname = part.get_payload(decode=True), part.get_filename()
            elif key:
                form[key] = part.get_payload(decode=True).decode("utf-8")
        if form.get(":action") != "file_upload" or content is None or not fname:
            return self.error(400, "not a file upload")
        fname = pathlib.PurePosixPath(fname.replace("\\", "/")).name
        if form.get("sha256_digest") and hashlib.sha256(content).hexdigest() != form["sha256_digest"]:
            return self.error(400, "%s: the file does not match its sha256_digest" % fname)
        d = self.root / "pypi" / normalize(form.get("name", ""))
        with LOCK:
            if (d / fname).exists():
                return self.error(409, "%s is already uploaded" % fname)
            d.mkdir(parents=True, exist_ok=True)
            (d / fname).write_bytes(content)
        self.send(200, b"OK", "text/plain")

    def pypi_get(self, path: str) -> None:
        pypi = self.root / "pypi"
        parts = [p for p in path.split("/") if p]
        if parts == ["simple"]:
            links = "".join('<a href="/simple/%s/">%s</a>\n' % (p.name, p.name) for p in sorted(pypi.iterdir()) if p.is_dir()) if pypi.is_dir() else ""
            return self.send(200, ("<!DOCTYPE html><html><body>\n%s</body></html>\n" % links).encode(), "text/html")
        if len(parts) == 2 and parts[0] == "simple":
            d = pypi / normalize(parts[1])
            if not d.is_dir():
                return self.error(404, "%s is not on the staging index" % parts[1])
            links = "".join(
                '<a href="/files/%s/%s#sha256=%s">%s</a>\n' % (d.name, html.escape(f.name), hashlib.sha256(f.read_bytes()).hexdigest(), html.escape(f.name))
                for f in sorted(d.iterdir()))
            return self.send(200, ("<!DOCTYPE html><html><body>\n%s</body></html>\n" % links).encode(), "text/html")
        if len(parts) == 3 and parts[0] == "files":
            f = pypi / normalize(parts[1]) / pathlib.PurePosixPath(urllib.parse.unquote(parts[2])).name
            return self.send(200, f.read_bytes(), "application/octet-stream") if f.is_file() else self.error(404, "no file %s" % parts[2])
        self.error(404, "not a simple-index path")

    def do_GET(self) -> None:
        path = self.path.split("?")[0]
        if path.startswith(("/simple", "/files/")):
            return self.pypi_get(path)
        if path.startswith("/-/"):  # npm's ping, audit and search endpoints
            return self.error(404, "not served here")
        return self.npm_get(path.lstrip("/"))

    do_HEAD = do_GET


class Server(ThreadingHTTPServer):
    def server_bind(self) -> None:
        # HTTPServer.server_bind resolves socket.getfqdn("127.0.0.1"), a reverse
        # DNS lookup that stalls past smoke.sh's 10 s wait on the macOS runners
        # (W-24); nothing here reads server_name, so the address stands in for it.
        socketserver.TCPServer.server_bind(self)
        host, port = self.server_address[:2]
        self.server_name, self.server_port = host, port


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("dir", type=pathlib.Path)
    ap.add_argument("--port-file", type=pathlib.Path)
    a = ap.parse_args()
    a.dir.mkdir(parents=True, exist_ok=True)
    Handler.root = a.dir.resolve()
    srv = Server(("127.0.0.1", 0), Handler)
    url = "http://127.0.0.1:%d" % srv.server_address[1]
    if a.port_file:
        tmp = a.port_file.with_suffix(".tmp")
        tmp.write_text(url, "utf-8")
        tmp.replace(a.port_file)  # whole or absent: a reader never sees half a URL
    sys.stderr.write("staging-index: serving %s on %s\n" % (Handler.root, url))
    srv.serve_forever()


if __name__ == "__main__":
    main()
