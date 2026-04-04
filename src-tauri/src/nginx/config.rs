use crate::db::DomainConfig;

/// Generate a complete nginx.conf from the list of domain configs.
pub fn generate(domains: &[DomainConfig], cert_dir: &str, nginx_dir: &str) -> String {
    let mut conf = format!(
        r#"worker_processes 1;
error_log  "{nginx_dir}/logs/error.log" warn;
pid        "{nginx_dir}/nginx.pid";

events {{
    worker_connections 256;
}}

http {{
    include      "{nginx_dir}/conf/mime.types";
    default_type  application/octet-stream;
    sendfile      on;
    keepalive_timeout 65;
    server_tokens off;

    # HTTP → redirect all to HTTPS
    server {{
        listen 80;
        server_name _;
        return 301 https://$host$request_uri;
    }}
"#,
        nginx_dir = nginx_dir.replace('\\', "/")
    );

    for d in domains.iter().filter(|d| d.enabled) {
        let cert_dir_fwd = cert_dir.replace('\\', "/");
        let cert = format!("{}/{}.crt", cert_dir_fwd, d.domain);
        let key = format!("{}/{}.key", cert_dir_fwd, d.domain);

        let adv = d.advanced_config.as_deref().unwrap_or("").trim();

        if adv.starts_with("server {") || adv.starts_with("server\n{") || adv.starts_with("server\r\n{") || adv.starts_with("server ") {
            let replaced = adv
                .replace("$DOMAIN", &d.domain)
                .replace("$CERT_PATH", &cert)
                .replace("$KEY_PATH", &key)
                .replace("$UPSTREAM", &d.upstream);
            conf.push_str("\n");
            conf.push_str(&replaced);
            conf.push_str("\n");
        } else {
            conf.push_str(&format!(
                r#"
    server {{
        listen 443 ssl;
        http2  on;
        server_name {domain};

        ssl_certificate     "{cert}";
        ssl_certificate_key "{key}";
        ssl_protocols       TLSv1.2 TLSv1.3;
        ssl_ciphers         HIGH:!aNULL:!MD5;
        ssl_session_cache   shared:SSL:1m;

        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header   Upgrade    $http_upgrade;
        proxy_set_header   Connection "upgrade";
        proxy_set_header   Host       $host;
        proxy_set_header   X-Real-IP  $remote_addr;
        proxy_read_timeout 300s;
        proxy_send_timeout 300s;

{advanced_config}

        location / {{
            proxy_pass {upstream};
        }}
    }}
"#,
                domain = d.domain,
                cert = cert,
                key = key,
                upstream = d.upstream,
                advanced_config = d.advanced_config.as_deref().unwrap_or(""),
            ));
        }
    }

    conf.push_str("}\n");
    conf
}
