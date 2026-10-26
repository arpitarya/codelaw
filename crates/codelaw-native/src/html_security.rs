//! `sec-html/*`: the HTML security rules of the `security-html` pack (W-56).
//!
//! Pattern-level, not taint analysis: each rule reads one start tag, one
//! attribute, one comment or one run of template syntax. A value that is
//! template code (`{{ }}`, `<% %>`) is only judged by its literal prefix.

use crate::html_dom::{finding, finding_with, is_dynamic, tokens_with_offsets, Attr, Elem};
use crate::html_rule;
use codelaw_core::TextEdit;
use regex::Regex;
use std::sync::LazyLock;

fn re(p: &str) -> Regex {
    Regex::new(p).expect("valid regex")
}

/// The offset just after the tag name of a start tag.
fn after_tag_name(src: &str, start: usize) -> usize {
    let rest = &src[start + 1..];
    let n = rest.find(|c: char| c.is_ascii_whitespace() || c == '>' || c == '/').unwrap_or(rest.len());
    start + 1 + n
}

/// A URL as a browser reads it for its scheme: leading whitespace and
/// control characters gone, tabs and newlines removed, lower-cased.
fn scheme_form(v: &str) -> String {
    v.trim_start_matches(|c: char| c.is_ascii_control() || c == ' ').chars().filter(|c| !matches!(c, '\t' | '\n' | '\r')).collect::<String>().to_ascii_lowercase()
}

fn loopback(url_lower: &str) -> bool {
    let rest = url_lower.split_once("//").map_or(url_lower, |(_, r)| r);
    let host = rest.split(['/', '?', '#', ':']).next().unwrap_or("");
    host == "localhost" || host.ends_with(".localhost") || host == "127.0.0.1" || rest.starts_with("[::1]")
}

/// `http://` to a host other than this machine.
fn is_http(v: &str) -> bool {
    let s = scheme_form(v);
    s.starts_with("http://") && !loopback(&s)
}

/// A URL that names another origin.
fn is_cross_origin(v: &str) -> bool {
    let s = scheme_form(v);
    s.starts_with("http://") || s.starts_with("https://") || s.starts_with("//")
}

/// The edit that turns an `http://` value into `https://`.
fn https_edit(a: &Attr) -> TextEdit {
    let v = a.value_or_empty();
    let lead = v.len() - v.trim_start_matches(|c: char| c.is_ascii_control() || c == ' ').len();
    TextEdit::new(a.value_start + lead, a.value_start + lead + 7, "https://")
}

fn plain(a: &Attr) -> bool {
    !a.value_or_empty().trim().is_empty()
}

// ------------------------------------------------------------------ handlers

const EVENT_HANDLERS: &[&str] = &[
    "onabort", "onafterprint", "onanimationend", "onanimationiteration", "onanimationstart", "onauxclick", "onbegin", "onbeforeinput", "onbeforeprint",
    "onbeforeunload", "onblur", "oncancel", "oncanplay", "oncanplaythrough", "onchange", "onclick", "onclose", "oncontextmenu", "oncopy", "oncuechange", "oncut",
    "ondblclick", "ondrag", "ondragend", "ondragenter", "ondragleave", "ondragover", "ondragstart", "ondrop", "ondurationchange", "onemptied", "onend", "onended",
    "onerror", "onfocus", "onfocusin", "onfocusout", "onformdata", "onfullscreenchange", "onhashchange", "oninput", "oninvalid", "onkeydown", "onkeypress",
    "onkeyup", "onlanguagechange", "onload", "onloadeddata", "onloadedmetadata", "onloadstart", "onmessage", "onmessageerror", "onmousedown", "onmouseenter",
    "onmouseleave", "onmousemove", "onmouseout", "onmouseover", "onmouseup", "onmousewheel", "onoffline", "ononline", "onpagehide", "onpageshow", "onpaste",
    "onpause", "onplay", "onplaying", "onpointercancel", "onpointerdown", "onpointerenter", "onpointerleave", "onpointermove", "onpointerout", "onpointerover",
    "onpointerup", "onpopstate", "onprogress", "onratechange", "onrepeat", "onreset", "onresize", "onscroll", "onsearch", "onsecuritypolicyviolation", "onseeked",
    "onseeking", "onselect", "onselectionchange", "onselectstart", "onslotchange", "onstalled", "onstorage", "onsubmit", "onsuspend", "ontimeupdate", "ontoggle",
    "ontouchcancel", "ontouchend", "ontouchmove", "ontouchstart", "ontransitionend", "onunhandledrejection", "onunload", "onvolumechange", "onwaiting", "onwheel",
];

html_rule!(NoInlineEventHandler, "sec-html/no-inline-event-handler", |dom, meta, ctx, out| {
    for e in &dom.elems {
        for a in e.attrs.iter().filter(|a| EVENT_HANDLERS.contains(&a.name.as_str())) {
            out.push(finding(a.start, a.end, format!("`{}` is an inline event handler; a strict Content Security Policy blocks it, so attach the listener with `addEventListener`", a.name)));
        }
    }
});

// ------------------------------------------------------------- javascript:

const SCRIPT_URL_ATTRS: &[&str] = &["href", "src", "action", "formaction", "xlink:href"];

html_rule!(NoJavascriptUrl, "sec-html/no-javascript-url", |dom, meta, ctx, out| {
    for e in &dom.elems {
        for a in &e.attrs {
            let named = SCRIPT_URL_ATTRS.contains(&a.name.as_str()) || (a.name == "data" && e.is("object"));
            if named && scheme_form(a.value_or_empty()).starts_with("javascript:") {
                out.push(finding(a.start, a.end, format!("`{}` is a `javascript:` URL, which runs script from markup; use a real URL or a button with a listener", a.name)));
            }
        }
    }
});

// ------------------------------------------------------------ mixed content

const MIXED_LINK_RELS: &[&str] = &["stylesheet", "preload", "modulepreload", "prefetch", "icon", "manifest"];

html_rule!(NoMixedContent, "sec-html/no-mixed-content", |dom, meta, ctx, out| {
    for e in &dom.elems {
        for a in &e.attrs {
            let hit = match a.name.as_str() {
                "src" | "poster" => true,
                "action" | "formaction" => true,
                "data" => e.is("object"),
                "href" if e.is("link") => e
                    .value_lower("rel")
                    .is_some_and(|rel| rel.split_ascii_whitespace().any(|t| MIXED_LINK_RELS.contains(&t))),
                "href" if e.is("a") || e.is("area") => e.has("download"),
                _ => false,
            };
            if hit && is_http(a.value_or_empty()) {
                out.push(finding_with(
                    a.start,
                    a.end,
                    format!("`{}` loads `{}` over plain HTTP; a network attacker can change it, so use `https://`", a.name, a.value_or_empty().trim()),
                    vec![https_edit(a)],
                ));
            }
        }
    }
});

// -------------------------------------------------------------- password form

fn handles_submit_in_script(e: &Elem) -> bool {
    e.attrs.iter().any(|a| {
        let n = a.name.as_str();
        n == "onsubmit" || n.contains("submit") && (n.starts_with('@') || n.starts_with('(') || n.starts_with("v-on:") || n.starts_with("x-on:") || n.starts_with("ng-") || n.starts_with("data-ng-"))
            || n.starts_with("hx-")
            || n == "data-remote"
    })
}

html_rule!(PasswordFormPost, "sec-html/password-form-post", |dom, meta, ctx, out| {
    for i in dom.by_tag("form") {
        let e = &dom.elems[i];
        let has_password = dom.descendants(i).into_iter().any(|j| dom.elems[j].is("input") && dom.elems[j].value_lower("type").as_deref() == Some("password"));
        if !has_password {
            continue;
        }
        let (s, t) = dom.tag_range(i);
        match e.attr("method") {
            Some(m) if m.is_dynamic() => {}
            Some(m) => {
                let v = m.value_or_empty().trim().to_ascii_lowercase();
                if v != "post" && v != "dialog" {
                    let edit = if m.value.is_some() { TextEdit::new(m.value_start, m.value_end, "post") } else { TextEdit::new(m.start, m.end, "method=\"post\"") };
                    out.push(finding_with(s, t, "This form sends a password with GET, so it lands in the URL, the history and server logs; use `method=\"post\"`", vec![edit]));
                }
            }
            None if !handles_submit_in_script(e) => {
                let at = after_tag_name(ctx.source, e.start);
                out.push(finding_with(
                    s,
                    t,
                    "This form sends a password and has no `method`, so the browser uses GET and puts it in the URL; use `method=\"post\"`",
                    vec![TextEdit::new(at, at, " method=\"post\"")],
                ));
            }
            None => {}
        }
        if let Some(a) = e.attr("action").filter(|a| is_http(a.value_or_empty())) {
            out.push(finding_with(a.start, a.end, "This form sends a password to an `http:` URL, in the clear; use `https://`", vec![https_edit(a)]));
        }
    }
});

// -------------------------------------------------------------- iframe sandbox

html_rule!(IframeSandbox, "sec-html/iframe-sandbox", |dom, meta, ctx, out| {
    for i in dom.by_tag("iframe") {
        let e = &dom.elems[i];
        let (s, t) = dom.tag_range(i);
        let Some(a) = e.attr("sandbox") else {
            let at = after_tag_name(ctx.source, e.start);
            out.push(finding_with(s, t, "<iframe> has no `sandbox`, so the page it holds runs with the same powers as any top-level page; add `sandbox` and list only what it needs", vec![TextEdit::new(at, at, " sandbox")]));
            continue;
        };
        if a.is_dynamic() {
            continue;
        }
        let toks = tokens_with_offsets(a.value_or_empty(), a.value_start);
        let find = |name: &str| toks.iter().find(|(_, _, tok)| tok.eq_ignore_ascii_case(name)).copied();
        if find("allow-scripts").is_some()
            && let Some((ts, te, _)) = find("allow-same-origin")
        {
            let src = ctx.source;
            let before = src[..ts].trim_end_matches([' ', '\t', '\n', '\r']).len();
            let edit = if before > a.value_start {
                TextEdit::new(before, te, "")
            } else {
                let after = te + src[te..].len() - src[te..].trim_start_matches([' ', '\t', '\n', '\r']).len();
                TextEdit::new(ts, after.min(a.value_end), "")
            };
            out.push(finding_with(a.start, a.end, "`allow-scripts` with `allow-same-origin` lets the framed page remove its own sandbox; drop `allow-same-origin`", vec![edit]));
        }
    }
});

// ------------------------------------------------------------------ CSP in meta

html_rule!(CspMetaUnsafe, "sec-html/csp-meta-unsafe", |dom, meta, ctx, out| {
    for i in dom.by_tag("meta") {
        let e = &dom.elems[i];
        if e.value_lower("http-equiv").as_deref() != Some("content-security-policy") {
            continue;
        }
        let Some(c) = e.attr("content").filter(|c| !c.is_dynamic()) else { continue };
        let mut script: Option<Vec<String>> = None;
        let mut default: Option<Vec<String>> = None;
        for d in c.value_or_empty().split(';') {
            let mut it = d.split_ascii_whitespace();
            let Some(name) = it.next() else { continue };
            let sources: Vec<String> = it.map(str::to_ascii_lowercase).collect();
            match name.to_ascii_lowercase().as_str() {
                "script-src" | "script-src-elem" => script.get_or_insert_with(Vec::new).extend(sources),
                "default-src" => default = Some(sources),
                _ => {}
            }
        }
        let Some(sources) = script.or(default) else { continue };
        let guarded = sources.iter().any(|s| s.starts_with("'nonce-") || s.starts_with("'sha256-") || s.starts_with("'sha384-") || s.starts_with("'sha512-") || s == "'strict-dynamic'");
        let mut bad: Vec<&str> = Vec::new();
        for s in &sources {
            let hit = match s.as_str() {
                "'unsafe-inline'" => !guarded,
                "'unsafe-eval'" | "*" | "data:" => true,
                _ => false,
            };
            if hit && !bad.contains(&s.as_str()) {
                bad.push(s);
            }
        }
        if !bad.is_empty() {
            out.push(finding(c.start, c.end, format!("This Content Security Policy lets scripts load from `{}`, which defeats it; use nonces or hashes", bad.join("`, `"))));
        }
    }
});

// ------------------------------------------------------------------ <base>

html_rule!(NoExternalBase, "sec-html/no-external-base", |dom, meta, ctx, out| {
    for i in dom.by_tag("base") {
        if let Some(a) = dom.elems[i].attr("href").filter(|a| is_cross_origin(a.value_or_empty())) {
            out.push(finding(a.start, a.end, "`<base href>` points at another origin, so every relative URL on the page resolves there; use a path or drop the `<base>`"));
        }
    }
});

// --------------------------------------------------------------------- plugins

html_rule!(NoPluginEmbed, "sec-html/no-plugin-embed", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        if !matches!(e.tag.as_str(), "object" | "embed" | "applet") {
            continue;
        }
        if e.value_lower("type").is_some_and(|t| t.starts_with("image/")) {
            continue;
        }
        out.push(dom.at_elem(i, format!("<{}> runs a plugin or an external document with the page's trust; use <img>, <video>, <audio> or a sandboxed <iframe>", e.tag)));
    }
});

// --------------------------------------------------------- credentials in URLs

const URL_ATTRS: &[&str] = &["href", "src", "action", "formaction", "data", "poster", "cite", "longdesc", "background", "xlink:href", "ping"];

static CREDENTIALS: LazyLock<Regex> = LazyLock::new(|| re(r"^\s*(?:[A-Za-z][A-Za-z0-9+.\-]*:)?//[^/?#@\s]+:[^/?#@\s]*@"));

html_rule!(NoCredentialsInUrl, "sec-html/no-credentials-in-url", |dom, meta, ctx, out| {
    for e in &dom.elems {
        for a in e.attrs.iter().filter(|a| URL_ATTRS.contains(&a.name.as_str())) {
            if CREDENTIALS.is_match(a.value_or_empty()) {
                out.push(finding(a.start, a.end, format!("`{}` has a username and password in the URL; they show in history, logs and referrers, so keep credentials out of markup", a.name)));
            }
        }
    }
});

// ----------------------------------------------------------- template HTML sinks

const HTML_SINKS: &[&str] = &["[innerhtml]", "[outerhtml]", "bind-innerhtml", "bind-outerhtml", "v-html", "x-html", "ng-bind-html-unsafe"];

html_rule!(NoTemplateHtmlSink, "sec-html/no-template-html-sink", |dom, meta, ctx, out| {
    for e in &dom.elems {
        for a in e.attrs.iter().filter(|a| HTML_SINKS.contains(&a.name.as_str())) {
            out.push(finding(a.start, a.end, format!("`{}` writes its value as HTML, so a value a user controls is cross-site scripting; bind text (`{{{{ }}}}`, `textContent`) or sanitise first", a.name)));
        }
    }
});

// --------------------------------------------------------------- referrer policy

const SAFE_REFERRER: &str = "strict-origin-when-cross-origin";

fn leaky_referrer(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "unsafe-url" | "no-referrer-when-downgrade")
}

html_rule!(ReferrerPolicy, "sec-html/referrer-policy", |dom, meta, ctx, out| {
    for e in &dom.elems {
        let attr = if e.is("meta") && e.value_lower("name").as_deref() == Some("referrer") { e.attr("content") } else { e.attr("referrerpolicy") };
        if let Some(a) = attr.filter(|a| !a.is_dynamic() && leaky_referrer(a.value_or_empty())) {
            out.push(finding_with(
                a.start,
                a.end,
                format!("`{}` sends the full URL of this page, query string included, to other sites; use `{SAFE_REFERRER}`", a.value_or_empty().trim()),
                vec![TextEdit::new(a.value_start, a.value_end, SAFE_REFERRER)],
            ));
        }
    }
});

// ------------------------------------------------------------------- srcdoc

fn decode_entities(s: &str) -> String {
    let mut out = s.to_ascii_lowercase();
    for (from, to) in [("&lt;", "<"), ("&#60;", "<"), ("&#x3c;", "<"), ("&gt;", ">"), ("&quot;", "\""), ("&#34;", "\""), ("&apos;", "'"), ("&#39;", "'"), ("&amp;", "&")] {
        out = out.replace(from, to);
    }
    out
}

static SRCDOC_ACTIVE: LazyLock<Regex> = LazyLock::new(|| re(r#"(?is)<\s*script\b|<[^>]*[\s"'/]on[a-z]+\s*=|javascript:"#));

html_rule!(NoSrcdocScript, "sec-html/no-srcdoc-script", |dom, meta, ctx, out| {
    for i in dom.by_tag("iframe") {
        let Some(a) = dom.elems[i].attr("srcdoc").filter(|a| !a.is_dynamic()) else { continue };
        if SRCDOC_ACTIVE.is_match(&decode_entities(a.value_or_empty())) {
            out.push(finding(a.start, a.end, "`srcdoc` holds a `<script>` or an event handler, which runs with this page's origin unless the iframe is sandboxed; move it to a separate file in a sandboxed iframe"));
        }
    }
});

// ------------------------------------------------------------- SRI + crossorigin

html_rule!(SriCrossorigin, "sec-html/sri-crossorigin", |dom, meta, ctx, out| {
    for i in dom.all() {
        let e = &dom.elems[i];
        let url = match e.tag.as_str() {
            "script" => e.attr("src"),
            "link" => e.attr("href"),
            _ => continue,
        };
        let Some(integrity) = e.attr("integrity").filter(|a| plain(a)) else { continue };
        if e.has("crossorigin") || !url.is_some_and(|u| is_cross_origin(u.value_or_empty())) {
            continue;
        }
        out.push(finding_with(
            integrity.start,
            integrity.end,
            "`integrity` on a file from another origin is not checked without `crossorigin`; add `crossorigin=\"anonymous\"`",
            vec![TextEdit::new(integrity.end, integrity.end, " crossorigin=\"anonymous\"")],
        ));
    }
});

// ------------------------------------------------------------ hidden secrets

static SECRET_NAME: LazyLock<Regex> = LazyLock::new(|| re(r"(?i)pass(?:word|wd)|\bpwd\b|secret|token|api[-_ ]?key|apikey|private[-_ ]?key|access[-_ ]?key"));
static CSRF_NAME: LazyLock<Regex> = LazyLock::new(|| re(r"(?i)csrf|xsrf|authenticity|nonce|antiforgery|requestverification|^_token$|viewstate|eventvalidation"));

html_rule!(NoSecretInHiddenInput, "sec-html/no-secret-in-hidden-input", |dom, meta, ctx, out| {
    for i in dom.by_tag("input") {
        let e = &dom.elems[i];
        if e.value_lower("type").as_deref() != Some("hidden") {
            continue;
        }
        let Some(name) = e.attr("name").map(|n| n.value_or_empty().trim().to_owned()) else { continue };
        if !SECRET_NAME.is_match(&name) || CSRF_NAME.is_match(&name) {
            continue;
        }
        let Some(v) = e.attr("value").filter(|v| plain(v) && !v.is_dynamic()) else { continue };
        out.push(finding(v.start, v.end, format!("The hidden input `{name}` holds a secret written into the page; anyone who can see the page can read it, so keep it on the server")));
    }
});

// ---------------------------------------------------------- sensitive comments

static COMMENT_SECRET: LazyLock<Regex> = LazyLock::new(|| {
    re(r"(?is)pass(?:word|wd)\s*[:=]|\bsecret\b|api[-_ ]?key|private key|credentials?\s*[:=]|\btoken\s*[:=]|bearer\s+[a-z0-9._\-]{8,}|\b(?:10\.\d{1,3}|192\.168|172\.(?:1[6-9]|2\d|3[01]))\.\d{1,3}\.\d{1,3}\b|\b[a-z0-9-]+(?:\.[a-z0-9-]+)*\.(?:internal|corp|intranet|lan)\b|\b(?:todo|fixme)\b.{0,40}\bremove\b|\bremove\s+(?:before|in|for)\s+(?:prod|release|deploy|launch)")
});

html_rule!(NoSensitiveComments, "sec-html/no-sensitive-comments", |dom, meta, ctx, out| {
    let src = ctx.source;
    for &(s, e) in &dom.comments {
        let text = &src[s..e];
        if text.starts_with("<!--[if") || text.starts_with("<!--<![") || !COMMENT_SECRET.is_match(text) {
            continue;
        }
        // A comment alone on its line goes with its line.
        let line_start = src[..s].rfind('\n').map_or(0, |k| k + 1);
        let line_end = src[e..].find('\n').map_or(src.len(), |k| e + k + 1);
        let alone = src[line_start..s].trim().is_empty() && src[e..line_end].trim().is_empty();
        let edit = if alone { TextEdit::new(line_start, line_end, "") } else { TextEdit::new(s, e, "") };
        out.push(finding_with(s, e, "This comment is sent to every visitor and holds a password, key, internal host or a note to remove it; delete it", vec![edit]));
    }
});

// ------------------------------------------------------------------ CSRF field

const CSRF_NAMES: &[&str] = &["csrf", "xsrf", "authenticity_token", "antiforgery", "csrfmiddlewaretoken", "__requestverificationtoken", "_token"];

html_rule!(FormCsrfToken, "sec-html/form-csrf-token", |dom, meta, ctx, out| {
    let extra: Vec<String> = ctx.options.get("names").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_ascii_lowercase)).collect()).unwrap_or_default();
    for i in dom.by_tag("form") {
        let e = &dom.elems[i];
        if e.value_lower("method").as_deref() != Some("post") {
            continue;
        }
        // A form that posts to another site holds no token of this one.
        if e.attr("action").is_some_and(|a| is_cross_origin(a.value_or_empty())) {
            continue;
        }
        let end = e.end.min(ctx.source.len());
        let text = ctx.source[e.start..end].to_ascii_lowercase();
        let named = CSRF_NAMES.iter().any(|n| text.contains(n)) || extra.iter().any(|n| text.contains(n.as_str()));
        if !named {
            let (s, t) = dom.tag_range(i);
            out.push(finding(s, t, "This POST form carries no CSRF token, so another site can submit it as the signed-in user; add the framework's hidden token field"));
        }
    }
});

// -------------------------------------------------------------- AngularJS URLs

html_rule!(AngularjsUrlExpression, "sec-html/angularjs-url-expression", |dom, meta, ctx, out| {
    let angular = dom.elems.iter().any(|e| {
        e.attrs.iter().any(|a| a.name.starts_with("ng-") || a.name.starts_with("data-ng-") || a.name.starts_with("x-ng-"))
            || (e.is("script") && e.attr("src").is_some_and(|s| s.value_or_empty().to_ascii_lowercase().contains("angular")))
    });
    if !angular {
        return;
    }
    for e in &dom.elems {
        for a in e.attrs.iter().filter(|a| matches!(a.name.as_str(), "href" | "src") && a.value_or_empty().contains("{{")) {
            let to = if a.name == "href" { "ng-href" } else { "ng-src" };
            out.push(finding_with(
                a.start,
                a.end,
                format!("`{}=\"{{{{…}}}}\"` is requested before AngularJS fills it in; use `{to}`", a.name),
                vec![TextEdit::new(a.name_start, a.name_end, to)],
            ));
        }
    }
});

// ------------------------------------------------- unescaped template output

static TRIPLE_STACHE: LazyLock<Regex> = LazyLock::new(|| re(r"\{\{\{[^{}\n]+\}\}\}"));
static JINJA_SAFE: LazyLock<Regex> = LazyLock::new(|| re(r"\{\{(?P<e>[^{}]*?)\|\s*safe\b[^{}]*\}\}"));
static AUTOESCAPE_OFF: LazyLock<Regex> = LazyLock::new(|| re(r"\{%-?\s*autoescape\s+(?:off|false)\s*-?%\}"));
static EJS_RAW: LazyLock<Regex> = LazyLock::new(|| re(r"<%-(?P<e>[^%]*?)(?P<c>-?)%>"));

html_rule!(NoUnsafeTemplateInterpolation, "sec-html/no-unsafe-template-interpolation", |dom, meta, ctx, out| {
    let src = ctx.source;
    let msg = |what: &str| format!("{what} writes its value without escaping, so a value a user controls is cross-site scripting; let the template escape it, or sanitise it first");
    for m in TRIPLE_STACHE.find_iter(src) {
        if m.start() > 0 && src.as_bytes()[m.start() - 1] == b'{' {
            continue;
        }
        out.push(finding(m.start(), m.end(), msg("`{{{ }}}`")));
    }
    for c in JINJA_SAFE.captures_iter(src) {
        let m = c.get(0).expect("whole match");
        if c["e"].trim_start().starts_with(['"', '\'']) {
            continue;
        }
        out.push(finding(m.start(), m.end(), msg("`|safe`")));
    }
    for m in AUTOESCAPE_OFF.find_iter(src) {
        out.push(finding(m.start(), m.end(), msg("`autoescape off`")));
    }
    const RUBY: &[&str] = &["if", "unless", "else", "elsif", "end", "each", "for", "while", "case", "when", "do", "begin", "include"];
    for c in EJS_RAW.captures_iter(src) {
        let m = c.get(0).expect("whole match");
        let first = c["e"].split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_').find(|w| !w.is_empty()).unwrap_or("");
        // `-%>` is ERB's trim mark, and `<%- include(…) %>` pulls a partial.
        if !c["c"].is_empty() || RUBY.contains(&first) || is_dynamic(&c["e"]) {
            continue;
        }
        out.push(finding(m.start(), m.end(), msg("`<%-`")));
    }
});

// ------------------------------------------------------------ powerful iframes

const POWERFUL: &[&str] = &["camera", "microphone", "geolocation", "payment", "display-capture", "usb", "serial", "hid", "bluetooth"];

html_rule!(IframeAllowPowerful, "sec-html/iframe-allow-powerful", |dom, meta, ctx, out| {
    for i in dom.by_tag("iframe") {
        let e = &dom.elems[i];
        let (Some(allow), Some(src)) = (e.attr("allow").filter(|a| !a.is_dynamic()), e.attr("src")) else { continue };
        if !is_cross_origin(src.value_or_empty()) {
            continue;
        }
        let mut granted: Vec<String> = Vec::new();
        for d in allow.value_or_empty().split(';') {
            let mut it = d.split_ascii_whitespace();
            let Some(feature) = it.next().map(str::to_ascii_lowercase) else { continue };
            let list: Vec<&str> = it.collect();
            let withheld = list.len() == 1 && matches!(list[0], "'none'" | "'self'");
            if POWERFUL.contains(&feature.as_str()) && !withheld {
                granted.push(feature);
            }
        }
        if !granted.is_empty() {
            out.push(finding(allow.start, allow.end, format!("This iframe from another origin may use the {}; grant it only to a framed page you trust, or drop it from `allow`", granted.join(", "))));
        }
    }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html_dom::tests::run;
    use crate::NativeRule;

    fn n(rule: &dyn NativeRule, src: &str) -> usize {
        run(rule, src).len()
    }

    fn fixed(rule: &dyn NativeRule, src: &str) -> String {
        let mut edits: Vec<TextEdit> = run(rule, src).into_iter().flat_map(|f| f.edits).collect();
        edits.sort_by_key(|e| std::cmp::Reverse(e.start));
        let mut s = src.to_owned();
        for e in edits {
            s.replace_range(e.start..e.end, &e.text);
        }
        s
    }

    #[test]
    fn handlers_and_script_urls() {
        assert_eq!(n(&NoInlineEventHandler, r#"<button onclick="go()" onmouseover="x()">a</button>"#), 2);
        assert_eq!(n(&NoInlineEventHandler, r#"<div data-onclick="a" (click)="b" @click="c" onboarding="d"></div>"#), 0);
        assert_eq!(n(&NoJavascriptUrl, r#"<a href=" JavaScript:alert(1)">a</a><form action="java&#9;script:x"></form>"#), 1);
        assert_eq!(n(&NoJavascriptUrl, "<a href=\"java\tscript:alert(1)\">a</a><button formaction=\"javascript:x()\"></button>"), 2);
        assert_eq!(n(&NoJavascriptUrl, r#"<object data="javascript:x"></object><div data="javascript:x"></div>"#), 1);
        assert_eq!(n(&NoJavascriptUrl, r#"<a href="/js">a</a>"#), 0);
    }

    #[test]
    fn mixed_content() {
        let src = r#"<script src="http://a.example/x.js"></script><img src="https://a/b.png"><img src="http://localhost:3000/a.png">"#;
        assert_eq!(n(&NoMixedContent, src), 1);
        assert_eq!(fixed(&NoMixedContent, r#"<img src="HTTP://a.example/p.png">"#), r#"<img src="https://a.example/p.png">"#);
        assert_eq!(n(&NoMixedContent, r#"<a href="http://a.example/x">a</a><a href="http://a.example/x.pdf" download>b</a>"#), 1);
        assert_eq!(n(&NoMixedContent, r#"<link rel="stylesheet" href="http://a/s.css"><link rel="canonical" href="http://a/">"#), 1);
        assert_eq!(n(&NoMixedContent, r#"<form action="http://a.example/p"></form>"#), 1);
    }

    #[test]
    fn password_forms() {
        assert_eq!(n(&PasswordFormPost, r#"<form method="get"><input type="password" name="p"></form>"#), 1);
        assert_eq!(fixed(&PasswordFormPost, r#"<form method="GET"><input type="password"></form>"#), r#"<form method="post"><input type="password"></form>"#);
        assert_eq!(fixed(&PasswordFormPost, r#"<form action="/x"><input type="password"></form>"#), r#"<form method="post" action="/x"><input type="password"></form>"#);
        assert_eq!(n(&PasswordFormPost, r#"<form method="post"><input type="password"></form>"#), 0);
        assert_eq!(n(&PasswordFormPost, r#"<form (ngSubmit)="go()"><input type="password"></form>"#), 0);
        assert_eq!(n(&PasswordFormPost, r#"<form method="post" action="http://a.example/l"><input type="password"></form>"#), 1);
        assert_eq!(n(&PasswordFormPost, r#"<form method="get"><input type="text"></form>"#), 0);
    }

    #[test]
    fn iframes() {
        assert_eq!(fixed(&IframeSandbox, r#"<iframe src="/a"></iframe>"#), r#"<iframe sandbox src="/a"></iframe>"#);
        assert_eq!(n(&IframeSandbox, r#"<iframe sandbox src="/a"></iframe>"#), 0);
        assert_eq!(n(&IframeSandbox, r#"<iframe sandbox="allow-forms" src="/a"></iframe>"#), 0);
        assert_eq!(fixed(&IframeSandbox, r#"<iframe sandbox="allow-scripts allow-same-origin" src="/a"></iframe>"#), r#"<iframe sandbox="allow-scripts" src="/a"></iframe>"#);
        assert_eq!(fixed(&IframeSandbox, r#"<iframe sandbox="allow-same-origin allow-scripts"></iframe>"#), r#"<iframe sandbox="allow-scripts"></iframe>"#);
        assert_eq!(n(&IframeAllowPowerful, r#"<iframe src="https://a.example/e" allow="camera; microphone 'none'; fullscreen"></iframe>"#), 1);
        assert_eq!(n(&IframeAllowPowerful, r#"<iframe src="/e" allow="camera"></iframe>"#), 0);
        assert_eq!(n(&IframeAllowPowerful, r#"<iframe src="https://a.example/e" allow="camera 'none'"></iframe>"#), 0);
        assert_eq!(n(&NoSrcdocScript, r#"<iframe srcdoc="&lt;script&gt;x()&lt;/script&gt;"></iframe>"#), 1);
        assert_eq!(n(&NoSrcdocScript, r#"<iframe srcdoc="&lt;p onclick=x()&gt;a&lt;/p&gt;"></iframe>"#), 1);
        assert_eq!(n(&NoSrcdocScript, r#"<iframe srcdoc="&lt;p&gt;hello&lt;/p&gt;"></iframe>"#), 0);
    }

    #[test]
    fn csp_meta() {
        let m = |c: &str| format!(r#"<meta http-equiv="Content-Security-Policy" content="{c}">"#);
        assert_eq!(n(&CspMetaUnsafe, &m("script-src 'self' 'unsafe-inline' 'unsafe-eval'")), 1);
        assert_eq!(n(&CspMetaUnsafe, &m("default-src *")), 1);
        assert_eq!(n(&CspMetaUnsafe, &m("script-src 'self' data:")), 1);
        assert_eq!(n(&CspMetaUnsafe, &m("script-src 'self'; style-src 'unsafe-inline'")), 0);
        assert_eq!(n(&CspMetaUnsafe, &m("script-src 'nonce-abc' 'unsafe-inline'")), 0);
        assert_eq!(n(&CspMetaUnsafe, &m("script-src 'self'; img-src *")), 0);
    }

    #[test]
    fn base_plugins_credentials_sinks() {
        assert_eq!(n(&NoExternalBase, r#"<base href="https://evil.example/">"#), 1);
        assert_eq!(n(&NoExternalBase, r#"<base href="/app/">"#), 0);
        assert_eq!(n(&NoPluginEmbed, r#"<object data="a.swf"></object><embed src="a.swf"><applet code="A"></applet><object type="image/svg+xml" data="a.svg"></object>"#), 3);
        assert_eq!(n(&NoCredentialsInUrl, r#"<a href="https://user:pw@host.example/">a</a><a href="https://user@host.example/">b</a>"#), 1);
        assert_eq!(n(&NoCredentialsInUrl, r#"<img src="//u:p@h.example/a.png">"#), 1);
        assert_eq!(n(&NoTemplateHtmlSink, r#"<div [innerHTML]="a"></div><p v-html="b"></p><i [textContent]="c"></i>"#), 2);
    }

    #[test]
    fn referrer_and_sri() {
        assert_eq!(fixed(&ReferrerPolicy, r#"<a href="/a" referrerpolicy="unsafe-url">a</a>"#), r#"<a href="/a" referrerpolicy="strict-origin-when-cross-origin">a</a>"#);
        assert_eq!(n(&ReferrerPolicy, r#"<meta name="referrer" content="no-referrer-when-downgrade">"#), 1);
        assert_eq!(n(&ReferrerPolicy, r#"<meta name="referrer" content="no-referrer">"#), 0);
        let s = r#"<script src="https://cdn.example/a.js" integrity="sha384-abc"></script>"#;
        assert_eq!(fixed(&SriCrossorigin, s), r#"<script src="https://cdn.example/a.js" integrity="sha384-abc" crossorigin="anonymous"></script>"#);
        assert_eq!(n(&SriCrossorigin, r#"<script src="/a.js" integrity="sha384-abc"></script>"#), 0);
        assert_eq!(n(&SriCrossorigin, r#"<script src="https://cdn.example/a.js" integrity="sha384-abc" crossorigin></script>"#), 0);
    }

    #[test]
    fn hidden_secrets_and_comments() {
        assert_eq!(n(&NoSecretInHiddenInput, r#"<input type="hidden" name="api_key" value="sk_live_1">"#), 1);
        assert_eq!(n(&NoSecretInHiddenInput, r#"<input type="hidden" name="csrf_token" value="abc"><input type="hidden" name="_token" value="abc">"#), 0);
        assert_eq!(n(&NoSecretInHiddenInput, r#"<input type="hidden" name="api_key" value="{{ key }}"><input type="hidden" name="api_key" value="">"#), 0);
        assert_eq!(fixed(&NoSensitiveComments, "<p>a</p>\n  <!-- password: hunter2 -->\n<p>b</p>\n"), "<p>a</p>\n<p>b</p>\n");
        assert_eq!(fixed(&NoSensitiveComments, "<p>a</p><!-- TODO remove this -->"), "<p>a</p>");
        assert_eq!(n(&NoSensitiveComments, "<!-- the API is at 10.0.3.7 -->"), 1);
        assert_eq!(n(&NoSensitiveComments, "<!-- navigation -->"), 0);
        assert_eq!(n(&NoSensitiveComments, "<!--[if IE]><p>password: x</p><![endif]-->"), 0);
    }

    #[test]
    fn csrf_and_templates() {
        assert_eq!(n(&FormCsrfToken, r#"<form method="post" action="/a"><input name="x"></form>"#), 1);
        assert_eq!(n(&FormCsrfToken, r#"<form method="post"><input type="hidden" name="csrfmiddlewaretoken" value="x"></form>"#), 0);
        assert_eq!(n(&FormCsrfToken, r#"<form method="post">{% csrf_token %}</form>"#), 0);
        assert_eq!(n(&FormCsrfToken, r#"<form method="post" action="https://pay.example/"><input name="x"></form>"#), 0);
        assert_eq!(n(&FormCsrfToken, r#"<form method="get"><input name="q"></form>"#), 0);
        let ng = r#"<div ng-app><a href="{{url}}">a</a><img src="{{ p }}"></div>"#;
        assert_eq!(fixed(&AngularjsUrlExpression, ng), r#"<div ng-app><a ng-href="{{url}}">a</a><img ng-src="{{ p }}"></div>"#);
        assert_eq!(n(&AngularjsUrlExpression, r#"<a href="{{ url_for('x') }}">a</a>"#), 0);
        assert_eq!(n(&NoUnsafeTemplateInterpolation, "<p>{{{ body }}}</p><p>{{ x|safe }}</p><p>{{ \"<b>\"|safe }}</p>{% autoescape off %}<%- user %>"), 4);
        assert_eq!(n(&NoUnsafeTemplateInterpolation, "<%- include('a') %><%- if x -%><p>{{ x }}</p>{{{{raw}}}}"), 0);
    }
}
