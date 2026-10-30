type List = &'static [&'static str];

// (hosts, path markers ("" = any path), marker is the whole path, params to keep)
#[rustfmt::skip]
const PATH_RULES: &[(List, List, bool, List)] = &[
    (&["instagram.com"], &["/p/", "/reel/", "/reels/", "/tv/"], false, &["img_index"]),
    (&["x.com", "twitter.com"], &["/status/"], false, &[]),
    (&["youtu.be"], &[""], false, &["t"]),
    (&["youtube.com"], &["/shorts/"], false, &[]),
    (&["youtube.com"], &["/watch", "/watch/"], true, &["v", "t", "list", "index"]),
    (&["tiktok.com"], &["/video/"], false, &[]),
    (&["threads.net", "threads.com"], &["/post/"], false, &[]),
    (&["reddit.com"], &["/comments/"], false, &[]),
];

const DENYLIST: List = &[
    "fbclid", "gclid", "gclsrc", "dclid", "gbraid", "wbraid", "msclkid", "ttclid", "twclid",
    "igsh", "igshid", "mc_cid", "mc_eid", "ref_src", "ref_url", "_ga", "_gl", "si",
];

pub fn clean(text: &str) -> Option<String> {
    let url = text.trim();
    if url.contains(|c: char| c.is_whitespace() || c.is_control()) {
        return None;
    }
    let (head, fragment) = url.split_at(url.find('#').unwrap_or(url.len()));
    let (base, query) = head.split_once('?')?;
    let (scheme, rest) = base.split_once("://")?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return None;
    }
    let (authority, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let host_port = authority.rsplit('@').next()?;
    let (host, port) = host_port.split_once(':').unwrap_or((host_port, ""));
    // Malformed authority or dot segments: a browser would read the URL differently, so leave it.
    if host.is_empty()
        || !port.bytes().all(|b| b.is_ascii_digit())
        || !(port.is_empty() || port.parse::<u16>().is_ok())
        || path
            .split('/')
            .any(|seg| matches!(decoded_lower(seg).as_str(), "." | ".."))
    {
        return None;
    }
    let host = host.to_ascii_lowercase();

    let rule = PATH_RULES.iter().find(|(hosts, markers, exact, _)| {
        // Subdomain match already covers www./m./mobile., so they need no stripping.
        hosts
            .iter()
            .any(|d| host == *d || host.strip_suffix(d).is_some_and(|s| s.ends_with('.')))
            && markers
                .iter()
                .any(|m| if *exact { path == *m } else { path.contains(m) })
    });

    let kept: Vec<&str> = query
        .split('&')
        .filter(|param| {
            let name = decoded_lower(param.split('=').next().unwrap_or_default());
            match rule {
                Some((_, _, _, keep)) => keep.contains(&name.as_str()),
                None => !name.starts_with("utm_") && !DENYLIST.contains(&name.as_str()),
            }
        })
        .collect();

    let query = kept.join("&");
    let cleaned = if query.is_empty() {
        format!("{base}{fragment}")
    } else {
        format!("{base}?{query}{fragment}")
    };
    (cleaned != url).then_some(cleaned)
}

// Compares names as a browser would read them (%76 is v) while the output keeps the original bytes.
fn decoded_lower(name: &str) -> String {
    let hex = |b: &u8| (*b as char).to_digit(16).map(|d| d as u8);
    let (bytes, mut out, mut i) = (name.as_bytes(), Vec::with_capacity(name.len()), 0);
    while i < bytes.len() {
        match (
            bytes[i],
            bytes.get(i + 1).and_then(hex),
            bytes.get(i + 2).and_then(hex),
        ) {
            (b'%', Some(hi), Some(lo)) => {
                out.push(hi * 16 + lo);
                i += 3;
            }
            (b, _, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::clean;

    #[test]
    fn strips_tracking() {
        for (input, want) in [
            (
                "https://www.instagram.com/p/De9S54vjjrO/?utm_source=ig_web_copy_link&mdxt=NTc4MTIwNjQ2YQ==",
                "https://www.instagram.com/p/De9S54vjjrO/",
            ),
            (
                "https://www.instagram.com/reel/ABC123/?psln=MTIzNDU2Nzg5MA==",
                "https://www.instagram.com/reel/ABC123/",
            ),
            (
                "https://x.com/jimcramer/status/2116177685546189163?a=GkPAHMY28CmlFwUVB9g&s=20",
                "https://x.com/jimcramer/status/2116177685546189163",
            ),
            (
                "https://mobile.twitter.com/u/status/1?s=46&t=abc",
                "https://mobile.twitter.com/u/status/1",
            ),
            (
                "https://www.youtube.com/watch?v=abc&t=42s&si=xyz&feature=share",
                "https://www.youtube.com/watch?v=abc&t=42s",
            ),
            (
                "https://youtu.be/abc?si=xyz&t=10",
                "https://youtu.be/abc?t=10",
            ),
            (
                "https://www.youtube.com/shorts/abc?si=xyz",
                "https://www.youtube.com/shorts/abc",
            ),
            (
                "https://www.instagram.com/p/X/?img_index=2&igsh=abc",
                "https://www.instagram.com/p/X/?img_index=2",
            ),
            // Encoded names are compared decoded but written back untouched.
            (
                "https://www.youtube.com/watch?%76=abc&%74=42&si=xyz",
                "https://www.youtube.com/watch?%76=abc&%74=42",
            ),
            (
                "https://example.com/?%75tm_source=a&id=1",
                "https://example.com/?id=1",
            ),
            (
                "https://example.com/page?utm_source=a&id=5&fbclid=zz#frag",
                "https://example.com/page?id=5#frag",
            ),
            (
                "  https://x.com/u/status/1?s=20\n",
                "https://x.com/u/status/1",
            ),
            (
                "https://user:pw@EXAMPLE.com:8080/a?utm_medium=x&q=1",
                "https://user:pw@EXAMPLE.com:8080/a?q=1",
            ),
            ("https://x.com/u/status/1?", "https://x.com/u/status/1"),
            (
                "https://instagram.com/explore/?utm_source=x&foo=1",
                "https://instagram.com/explore/?foo=1",
            ),
            // Every remaining table rule.
            (
                "https://instagram.com/reels/a/?x=1",
                "https://instagram.com/reels/a/",
            ),
            (
                "https://instagram.com/tv/a/?x=1",
                "https://instagram.com/tv/a/",
            ),
            (
                "https://www.tiktok.com/@u/video/1?is_from_webapp=1",
                "https://www.tiktok.com/@u/video/1",
            ),
            (
                "https://www.threads.net/@u/post/a?xmt=1",
                "https://www.threads.net/@u/post/a",
            ),
            (
                "https://www.threads.com/@u/post/a?xmt=1",
                "https://www.threads.com/@u/post/a",
            ),
            (
                "https://old.reddit.com/r/a/comments/b/c/?share_id=1",
                "https://old.reddit.com/r/a/comments/b/c/",
            ),
            // Parsing edges the spec calls out.
            (
                "HTTPS://WWW.X.COM/u/status/1?s=20",
                "HTTPS://WWW.X.COM/u/status/1",
            ),
            (
                "https://example.com/?UTM_Source=a&FBCLID=b&id=5",
                "https://example.com/?id=5",
            ),
            (
                "https://www.youtube.com/watch?V=abc&si=x",
                "https://www.youtube.com/watch?V=abc",
            ),
            (
                "https://example.com/?fbclid&id=5",
                "https://example.com/?id=5",
            ),
            (
                "https://example.com/?q=a%20b&utm_source=x&r=%E2%9C%93",
                "https://example.com/?q=a%20b&r=%E2%9C%93",
            ),
            (
                "https://example.com/?名前=1&utm_source=a",
                "https://example.com/?名前=1",
            ),
            (
                "https://example.com/page?#frag?x=1",
                "https://example.com/page#frag?x=1",
            ),
            (
                "https://www.youtube.com/c/watchmojo?si=x&foo=1",
                "https://www.youtube.com/c/watchmojo?foo=1",
            ),
        ] {
            assert_eq!(clean(input).as_deref(), Some(want), "{input}");
        }
    }

    #[test]
    fn leaves_everything_else_alone() {
        for input in [
            "https://example.com/page?id=5",
            "https://example.com/page",
            "Check this https://x.com/u/status/1?s=20 out",
            "mailto:a@b.com",
            "https://notx.com/u/status/1?s=20",
            "",
            "日本語のテキスト",
            "ftp://example.com/?utm_source=a",
            "mailto:a@b.com?subject=hi&utm_source=a",
            "https://a.com/?utm_source=a https://b.com/?utm_source=b",
            "https://example.com/?utmost=1",
            "https://example.com/a#b?utm_source=x",
            "https://x.com@evil.com/status/1?s=20",
            "https://example.com/page?id=5\n",
            "https://www.youtube.com/watch_videos?video_ids=a,b",
            "https://www.youtube.com/shorts/../watch?v=abc&t=42",
            "https://www.youtube.com/shorts/./abc?si=x",
            "https://www.youtube.com/shorts/%2e%2e/watch?v=abc&t=42",
            "https://www.youtube.com/shorts/.%2E/watch?v=abc&t=42",
            "https://example.com:65536/?utm_source=x",
            "https://example.com:+80/?utm_source=x",
            "https://?utm_source=x",
            "https://example.com:abc/?utm_source=x",
            "https://example.com/?utm_source=x\u{0}",
            "https://[::1]:8080/?utm_source=x",
        ] {
            assert_eq!(clean(input), None, "{input}");
        }
    }
}
