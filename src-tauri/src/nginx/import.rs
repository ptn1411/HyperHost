use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, serde::Serialize)]
pub struct ImportedNginx {
    pub advanced_config: String,
    pub suggested_upstream: Option<String>,
    pub server_name: Option<String>,
}

/// Validate an advanced-config snippet by running `nginx -t` against a
/// wrapped harness config. Placeholders `$DOMAIN` / `$UPSTREAM` are substituted
/// with dummy values, `ssl_certificate*` directives and 443/SSL `listen` lines
/// are replaced so that the check only verifies syntax & directive validity.
pub fn validate_config(content: &str, nginx_exe: &Path) -> anyhow::Result<String> {
    let prepared = prepare_for_validation(content);

    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let test_dir: PathBuf = std::env::temp_dir().join(format!("hyperhost-validate-{}-{}", pid, ts));
    std::fs::create_dir_all(test_dir.join("logs"))?;
    std::fs::create_dir_all(test_dir.join("temp"))?;

    let conf_path = test_dir.join("nginx.conf");
    let pid_path = test_dir.join("nginx.pid");
    let error_log = test_dir.join("logs").join("error.log");

    let wrapped = format!(
        "worker_processes 1;\npid \"{pid}\";\nerror_log \"{err}\" info;\nevents {{ worker_connections 64; }}\nhttp {{\n    default_type application/octet-stream;\n    client_body_temp_path \"{tmp}/client\";\n    proxy_temp_path \"{tmp}/proxy\";\n    fastcgi_temp_path \"{tmp}/fastcgi\";\n    uwsgi_temp_path \"{tmp}/uwsgi\";\n    scgi_temp_path \"{tmp}/scgi\";\n{body}\n}}\n",
        pid = pid_path.to_string_lossy().replace('\\', "/"),
        err = error_log.to_string_lossy().replace('\\', "/"),
        tmp = test_dir.join("temp").to_string_lossy().replace('\\', "/"),
        body = indent_lines(&prepared, "    "),
    );

    std::fs::write(&conf_path, &wrapped)?;

    let prefix_str = test_dir.to_string_lossy().replace('\\', "/");
    let conf_str = conf_path.to_string_lossy().replace('\\', "/");

    let output = Command::new(nginx_exe)
        .args(["-t", "-c", &conf_str, "-p", &prefix_str])
        .output();

    let _ = std::fs::remove_dir_all(&test_dir);

    let output = output?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        Ok(stderr)
    } else {
        anyhow::bail!("{}", clean_validation_stderr(&stderr, &conf_str))
    }
}

fn prepare_for_validation(content: &str) -> String {
    let body = content
        .replace("$DOMAIN", "hyperhost-validate.test")
        .replace("$UPSTREAM", "http://127.0.0.1:65000");

    let mut out = String::new();
    for line in body.lines() {
        let t = line.trim();

        if t.starts_with("ssl_certificate")
            || t.starts_with("ssl_dhparam")
            || t.starts_with("ssl_trusted_certificate")
            || (t.starts_with("include") && t.contains("letsencrypt"))
        {
            continue;
        }

        if t.starts_with("listen") && (t.contains("443") || t.contains(" ssl") || t.contains("\tssl")) {
            let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
            out.push_str(&format!("{}listen 18080;\n", indent));
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }
    out
}

fn clean_validation_stderr(stderr: &str, conf_path: &str) -> String {
    stderr
        .lines()
        .filter(|l| !l.contains("[emerg]") || !l.contains("still could not"))
        .map(|l| l.replace(conf_path, "<config>"))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

pub fn convert_prod_to_dev(input: &str) -> Result<ImportedNginx, String> {
    let upstreams = extract_upstreams(input);
    let server_blocks = extract_top_blocks(input, "server");
    let https_block = server_blocks
        .iter()
        .find(|b| b.contains("ssl_certificate") || b.contains("listen 443") || b.contains("listen\t443"))
        .or_else(|| server_blocks.first())
        .ok_or("Không tìm thấy server block trong config")?;

    let server_name = find_directive_value(https_block, "server_name").map(|s| s.to_string());
    let port = find_main_upstream_port(https_block, &upstreams);
    let cleaned = clean_server_body(https_block, &upstreams);
    let advanced_config = build_server_template(&cleaned);

    Ok(ImportedNginx {
        advanced_config,
        suggested_upstream: port.map(|p| format!("http://127.0.0.1:{}", p)),
        server_name,
    })
}

pub fn convert_dev_to_prod(dev_config: &str, prod_domain: &str, prod_upstream: &str) -> String {
    let body = dev_config
        .replace("$DOMAIN", prod_domain)
        .replace("$UPSTREAM", prod_upstream);

    let mut out = String::new();
    let mut inside_server = false;
    let mut injected_listen = false;
    for line in body.lines() {
        let t = line.trim();

        if t.starts_with("server ") || t.starts_with("server\t") || t == "server{" || t.starts_with("server {") {
            inside_server = true;
            out.push_str(line);
            out.push('\n');
            continue;
        }

        if inside_server && !injected_listen && t.starts_with('{') {
            out.push_str(line);
            out.push('\n');
            out.push_str("    listen 80;\n    # Run: sudo certbot --nginx -d ");
            out.push_str(prod_domain);
            out.push('\n');
            injected_listen = true;
            continue;
        }

        if t.starts_with("ssl_certificate")
            || t.starts_with("ssl_protocols")
            || t.starts_with("ssl_ciphers")
            || t.starts_with("ssl_session")
            || t.starts_with("ssl_dhparam")
            || t.starts_with("listen 443")
            || t.starts_with("listen\t443")
            || t == "http2 on;"
            || t == "http2  on;"
            || (t.starts_with("include") && t.contains("letsencrypt"))
        {
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }
    out
}

fn extract_upstreams(input: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for block_with_header in extract_top_blocks_with_header(input, "upstream") {
        let (header, body) = block_with_header;
        let name = header
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .trim_matches(|c: char| c == '{' || c.is_whitespace())
            .to_string();
        if name.is_empty() {
            continue;
        }
        for line in body.lines() {
            let t = line.trim();
            if let Some(rest) = t.strip_prefix("server ") {
                let addr = rest
                    .split(|c: char| c.is_whitespace() || c == ';')
                    .find(|s| !s.is_empty())
                    .unwrap_or("");
                if !addr.is_empty() {
                    result.insert(name.clone(), addr.to_string());
                    break;
                }
            }
        }
    }
    result
}

fn extract_top_blocks(input: &str, keyword: &str) -> Vec<String> {
    extract_top_blocks_with_header(input, keyword)
        .into_iter()
        .map(|(_, body)| body)
        .collect()
}

fn extract_top_blocks_with_header(input: &str, keyword: &str) -> Vec<(String, String)> {
    let stripped = strip_comments(input);
    let bytes = stripped.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let remaining = &stripped[i..];
        let Some(pos) = find_keyword(remaining, keyword) else { break };
        let start = i + pos;
        let mut j = start + keyword.len();
        let mut header_end = j;
        while j < bytes.len() && bytes[j] != b'{' {
            j += 1;
            header_end = j;
        }
        if j >= bytes.len() {
            break;
        }
        let header = stripped[start..header_end].trim().to_string();
        let mut depth = 1;
        let block_start = j + 1;
        let mut k = block_start;
        while k < bytes.len() && depth > 0 {
            match bytes[k] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                k += 1;
            }
        }
        if depth != 0 {
            break;
        }
        let body = stripped[block_start..k].to_string();
        result.push((header, body));
        i = k + 1;
    }
    result
}

fn find_keyword(input: &str, keyword: &str) -> Option<usize> {
    let bytes = input.as_bytes();
    let klen = keyword.len();
    let mut i = 0;
    while i + klen <= bytes.len() {
        if &bytes[i..i + klen] == keyword.as_bytes() {
            let before_ok = i == 0 || matches!(bytes[i - 1], b' ' | b'\t' | b'\n' | b'\r' | b';' | b'}');
            let after = bytes.get(i + klen).copied();
            let after_ok = matches!(after, Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r') | Some(b'{'));
            if before_ok && after_ok {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

fn strip_comments(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for line in input.lines() {
        let mut in_quote = false;
        let mut trimmed_end = line.len();
        for (idx, ch) in line.char_indices() {
            match ch {
                '"' | '\'' => in_quote = !in_quote,
                '#' if !in_quote => {
                    trimmed_end = idx;
                    break;
                }
                _ => {}
            }
        }
        out.push_str(&line[..trimmed_end]);
        out.push('\n');
    }
    out
}

fn find_directive_value<'a>(block: &'a str, directive: &str) -> Option<&'a str> {
    for line in block.lines() {
        let t = line.trim();
        let Some(rest) = t.strip_prefix(directive) else { continue };
        if !rest.starts_with(' ') && !rest.starts_with('\t') {
            continue;
        }
        let value = rest.trim().trim_end_matches(';').trim();
        let first = value.split_whitespace().next()?;
        let start = line.find(first)?;
        return Some(&line[start..start + first.len()]);
    }
    None
}

fn find_main_upstream_port(block: &str, upstreams: &HashMap<String, String>) -> Option<u16> {
    if let Some(target) = find_location_proxy_pass(block, "/") {
        if let Some(p) = extract_port(&target, upstreams) {
            return Some(p);
        }
    }
    for line in block.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("proxy_pass") {
            let target = rest.trim().trim_end_matches(';').trim();
            if let Some(p) = extract_port(target, upstreams) {
                return Some(p);
            }
        }
    }
    None
}

fn find_location_proxy_pass(block: &str, path: &str) -> Option<String> {
    for (header, body) in extract_top_blocks_with_header(block, "location") {
        let header_path = header
            .trim_start_matches("location")
            .trim()
            .split_whitespace()
            .next()
            .unwrap_or("");
        if header_path == path {
            for line in body.lines() {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("proxy_pass") {
                    return Some(rest.trim().trim_end_matches(';').trim().to_string());
                }
            }
        }
    }
    None
}

fn extract_port(target: &str, upstreams: &HashMap<String, String>) -> Option<u16> {
    let stripped = target
        .trim_start_matches("http://")
        .trim_start_matches("https://");
    let host_port = stripped.split('/').next()?;
    if let Some(addr) = upstreams.get(host_port) {
        return addr.rsplit(':').next()?.trim().parse().ok();
    }
    host_port.rsplit_once(':').and_then(|(_, p)| p.parse().ok())
}

fn clean_server_body(block: &str, upstreams: &HashMap<String, String>) -> String {
    let body = block.trim();
    let mut out = String::new();
    for line in body.lines() {
        let t = line.trim();
        if t.starts_with("listen ") || t.starts_with("listen\t") {
            continue;
        }
        if t.starts_with("server_name ") || t.starts_with("server_name\t") {
            continue;
        }
        if t.starts_with("ssl_certificate")
            || t.starts_with("ssl_protocols")
            || t.starts_with("ssl_ciphers")
            || t.starts_with("ssl_dhparam")
            || t.starts_with("ssl_session")
        {
            continue;
        }
        if t.starts_with("include") && t.contains("letsencrypt") {
            continue;
        }
        if t.starts_with("access_log ") || t.starts_with("error_log ") {
            continue;
        }
        if t.starts_with("root ") {
            continue;
        }
        if t.starts_with("if (") && t.contains("$host") {
            continue;
        }
        if t.starts_with("return 301") && t.contains("https://") {
            continue;
        }

        if t.starts_with("proxy_pass ") || t.starts_with("proxy_pass\t") {
            out.push_str(&rewrite_proxy_pass(line, upstreams));
            out.push('\n');
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }
    squash_blank_lines(&out)
}

fn rewrite_proxy_pass(line: &str, upstreams: &HashMap<String, String>) -> String {
    let t = line.trim();
    let Some(rest) = t.strip_prefix("proxy_pass").or_else(|| t.strip_prefix("proxy_pass\t")) else {
        return line.to_string();
    };
    let target = rest.trim().trim_end_matches(';').trim();
    let new_target = rewrite_target(target, upstreams);
    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    format!("{}proxy_pass {};", indent, new_target)
}

fn rewrite_target(target: &str, upstreams: &HashMap<String, String>) -> String {
    let scheme = if target.starts_with("https://") {
        "https://"
    } else if target.starts_with("http://") {
        "http://"
    } else {
        return target.to_string();
    };
    let rest = &target[scheme.len()..];
    let (host_port, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };

    let is_main = upstreams.contains_key(host_port)
        || host_port == "localhost"
        || host_port.starts_with("localhost:")
        || host_port.starts_with("127.0.0.1");

    if is_main {
        format!("$UPSTREAM{}", path)
    } else {
        target.to_string()
    }
}

fn squash_blank_lines(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_blank = false;
    for line in s.lines() {
        let blank = line.trim().is_empty();
        if blank && prev_blank {
            continue;
        }
        out.push_str(line);
        out.push('\n');
        prev_blank = blank;
    }
    out
}

fn build_server_template(cleaned_body: &str) -> String {
    format!(
        "server {{\n    listen 443 ssl;\n    http2  on;\n    server_name $DOMAIN;\n\n    ssl_certificate     \"$CERT_PATH\";\n    ssl_certificate_key \"$KEY_PATH\";\n    ssl_protocols       TLSv1.2 TLSv1.3;\n    ssl_ciphers         HIGH:!aNULL:!MD5;\n    ssl_session_cache   shared:SSL:1m;\n\n    # --- Imported from prod ---\n{body}\n}}\n",
        body = indent_lines(cleaned_body.trim(), "    ")
    )
}

fn indent_lines(s: &str, prefix: &str) -> String {
    s.lines()
        .map(|l| {
            if l.trim().is_empty() {
                String::new()
            } else {
                format!("{}{}", prefix, l.trim_end())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_proxy_block() {
        let input = r#"
server {
    server_name hivemind.bug.edu.vn;
    location / {
        proxy_pass http://localhost:4210;
        proxy_set_header Host $host;
    }
    listen 443 ssl;
    ssl_certificate /etc/letsencrypt/live/x/fullchain.pem;
}
"#;
        let r = convert_prod_to_dev(input).unwrap();
        assert_eq!(r.server_name.as_deref(), Some("hivemind.bug.edu.vn"));
        assert_eq!(r.suggested_upstream.as_deref(), Some("http://127.0.0.1:4210"));
        assert!(r.advanced_config.contains("proxy_pass $UPSTREAM;"));
        assert!(!r.advanced_config.contains("letsencrypt"));
        assert!(!r.advanced_config.contains("listen 443 ssl;\n    listen"));
    }

    #[test]
    fn upstream_reference_resolves_port() {
        let input = r#"
upstream backend_api {
    server 127.0.0.1:3000;
}
server {
    server_name messages-api.bug.edu.vn;
    location /ws {
        proxy_pass http://backend_api;
    }
    location / {
        proxy_pass http://backend_api;
    }
    listen 443 ssl;
    ssl_certificate /x;
}
"#;
        let r = convert_prod_to_dev(input).unwrap();
        assert_eq!(r.suggested_upstream.as_deref(), Some("http://127.0.0.1:3000"));
        assert!(r.advanced_config.contains("proxy_pass $UPSTREAM;"));
    }
}
