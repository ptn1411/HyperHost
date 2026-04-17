use serde::Serialize;
use std::fs;
use std::path::Path;

const SKIP_DIRS: &[&str] = &[
    "node_modules", ".git", "target", "dist", ".next", "build",
    "vendor", "__pycache__", ".venv", "venv", ".cargo", ".svelte-kit",
    ".nuxt", ".output", "out", "coverage", ".turbo", ".parcel-cache",
    ".cache", "tmp", "logs", ".idea", ".vscode", ".DS_Store",
];

const MAX_RESULTS: usize = 200;

#[derive(Debug, Serialize)]
pub struct ProjectInfo {
    pub path: String,
    pub name: String,
    pub kind: String,
    pub suggested_port: u16,
    pub suggested_domain: String,
    pub suggested_command: Option<String>,
}

pub fn scan_projects(root: &Path, max_depth: usize) -> Vec<ProjectInfo> {
    let mut results = Vec::new();
    walk(root, 0, max_depth, &mut results);
    results
}

fn walk(dir: &Path, depth: usize, max_depth: usize, out: &mut Vec<ProjectInfo>) {
    if out.len() >= MAX_RESULTS {
        return;
    }
    if depth > max_depth {
        return;
    }

    if let Some(info) = detect(dir) {
        out.push(info);
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if name.starts_with('.') {
            continue;
        }
        if SKIP_DIRS.contains(&name) {
            continue;
        }
        walk(&path, depth + 1, max_depth, out);
    }
}

fn detect(dir: &Path) -> Option<ProjectInfo> {
    let name = dir.file_name()?.to_str()?.to_string();
    let path_str = dir.to_string_lossy().to_string();
    let domain = format!("{}.test", slugify(&name));

    let (kind, port) = detect_kind(dir)?;
    let command = suggested_command(&kind, dir);

    Some(ProjectInfo {
        path: path_str,
        name,
        kind,
        suggested_port: port,
        suggested_domain: domain,
        suggested_command: command,
    })
}

fn detect_kind(dir: &Path) -> Option<(String, u16)> {
    if dir.join("package.json").exists() {
        let pkg = fs::read_to_string(dir.join("package.json")).unwrap_or_default();
        let (kind, port) = detect_node_flavor(&pkg);
        return Some((kind.to_string(), port));
    }
    if dir.join("Cargo.toml").exists() {
        return Some(("rust".into(), 8080));
    }
    if dir.join("manage.py").exists() {
        return Some(("django".into(), 8000));
    }
    if dir.join("artisan").exists() {
        return Some(("laravel".into(), 8000));
    }
    if dir.join("composer.json").exists() {
        return Some(("php".into(), 8000));
    }
    if dir.join("config").join("application.rb").exists() || dir.join("Gemfile").exists() {
        return Some(("rails".into(), 3000));
    }
    if dir.join("go.mod").exists() {
        return Some(("go".into(), 8080));
    }
    if dir.join("pom.xml").exists()
        || dir.join("build.gradle").exists()
        || dir.join("build.gradle.kts").exists()
    {
        return Some(("java".into(), 8080));
    }
    if dir.join("pyproject.toml").exists() {
        let content = fs::read_to_string(dir.join("pyproject.toml")).unwrap_or_default();
        if content.contains("fastapi") {
            return Some(("fastapi".into(), 8000));
        }
        if content.contains("flask") {
            return Some(("flask".into(), 5000));
        }
        return Some(("python".into(), 8000));
    }
    if dir.join("requirements.txt").exists() {
        return Some(("python".into(), 8000));
    }
    if dir.join("mix.exs").exists() {
        return Some(("phoenix".into(), 4000));
    }
    if dir.join("Package.swift").exists() {
        return Some(("swift".into(), 8080));
    }
    None
}

fn suggested_command(kind: &str, dir: &Path) -> Option<String> {
    match kind {
        "nextjs" | "vite" | "nuxt" | "sveltekit" | "angular" | "astro" | "remix"
        | "cra" | "nestjs" | "strapi" | "expo" | "node" | "node-api" => {
            let pkg = fs::read_to_string(dir.join("package.json")).unwrap_or_default();
            if pkg.contains("\"dev\":") {
                Some("npm run dev".into())
            } else if pkg.contains("\"start\":") {
                Some("npm start".into())
            } else {
                Some("npm install".into())
            }
        }
        "rust" => Some("cargo run".into()),
        "django" => Some("python manage.py runserver".into()),
        "laravel" => Some("php artisan serve".into()),
        "php" => Some("php -S 127.0.0.1:8000".into()),
        "rails" => Some("bundle exec rails server".into()),
        "go" => Some("go run .".into()),
        "phoenix" => Some("mix phx.server".into()),
        "fastapi" => Some("uvicorn main:app --reload".into()),
        "flask" => Some("flask run".into()),
        "python" => Some("python main.py".into()),
        "swift" => Some("swift run".into()),
        "java" => {
            if dir.join("build.gradle").exists() || dir.join("build.gradle.kts").exists() {
                Some(if cfg!(target_os = "windows") {
                    "gradlew.bat bootRun".into()
                } else {
                    "./gradlew bootRun".into()
                })
            } else {
                Some("mvn spring-boot:run".into())
            }
        }
        _ => None,
    }
}

fn detect_node_flavor(pkg: &str) -> (&'static str, u16) {
    let has = |k: &str| pkg.contains(&format!("\"{}\"", k));
    if has("next") {
        return ("nextjs", 3000);
    }
    if has("nuxt") || has("nuxt3") {
        return ("nuxt", 3000);
    }
    if has("@angular/core") {
        return ("angular", 4200);
    }
    if has("astro") {
        return ("astro", 4321);
    }
    if has("@remix-run/react") || has("@remix-run/node") {
        return ("remix", 3000);
    }
    if has("@sveltejs/kit") {
        return ("sveltekit", 5173);
    }
    if has("vite") {
        return ("vite", 5173);
    }
    if has("react-scripts") {
        return ("cra", 3000);
    }
    if has("@strapi/strapi") || has("strapi") {
        return ("strapi", 1337);
    }
    if has("@nestjs/core") {
        return ("nestjs", 3000);
    }
    if has("express") || has("fastify") || has("koa") || has("hono") {
        return ("node-api", 3000);
    }
    if has("expo") {
        return ("expo", 19006);
    }
    ("node", 3000)
}

fn slugify(s: &str) -> String {
    let lower = s.to_lowercase();
    let mapped: String = lower
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();
    let trimmed = mapped.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "project".into()
    } else {
        trimmed
    }
}
