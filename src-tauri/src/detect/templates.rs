use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Template {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub default_upstream: &'static str,
    pub description: &'static str,
    pub advanced_config: Option<&'static str>,
}

pub fn all() -> Vec<Template> {
    vec![
        // ── Node / JS frontend ──
        Template { id: "nextjs",    name: "Next.js",       category: "Node/JS", default_upstream: "http://127.0.0.1:3000", description: "React framework, HMR via WebSocket.",   advanced_config: None },
        Template { id: "vite",      name: "Vite",          category: "Node/JS", default_upstream: "http://127.0.0.1:5173", description: "Vite dev server, HMR via WebSocket.",   advanced_config: None },
        Template { id: "nuxt",      name: "Nuxt",          category: "Node/JS", default_upstream: "http://127.0.0.1:3000", description: "Vue/Nuxt SSR, HMR via WebSocket.",       advanced_config: None },
        Template { id: "sveltekit", name: "SvelteKit",     category: "Node/JS", default_upstream: "http://127.0.0.1:5173", description: "SvelteKit dev server.",                  advanced_config: None },
        Template { id: "angular",   name: "Angular",       category: "Node/JS", default_upstream: "http://127.0.0.1:4200", description: "Angular CLI ng serve.",                  advanced_config: None },
        Template { id: "astro",     name: "Astro",         category: "Node/JS", default_upstream: "http://127.0.0.1:4321", description: "Astro dev server.",                      advanced_config: None },
        Template { id: "remix",     name: "Remix",         category: "Node/JS", default_upstream: "http://127.0.0.1:3000", description: "Remix dev server.",                      advanced_config: None },
        Template { id: "storybook", name: "Storybook",     category: "Node/JS", default_upstream: "http://127.0.0.1:6006", description: "Component workshop, HMR via WebSocket.", advanced_config: None },

        // ── Node backends ──
        Template { id: "express",   name: "Express / Node",category: "Backend", default_upstream: "http://127.0.0.1:3000", description: "Plain Node/Express/Fastify API.",        advanced_config: None },
        Template { id: "nestjs",    name: "NestJS",        category: "Backend", default_upstream: "http://127.0.0.1:3333", description: "NestJS default port.",                   advanced_config: None },
        Template { id: "strapi",    name: "Strapi",        category: "Backend", default_upstream: "http://127.0.0.1:1337", description: "Strapi headless CMS.",                   advanced_config: None },

        // ── Python ──
        Template { id: "django",    name: "Django",        category: "Python",  default_upstream: "http://127.0.0.1:8000", description: "python manage.py runserver.",            advanced_config: None },
        Template { id: "fastapi",   name: "FastAPI",       category: "Python",  default_upstream: "http://127.0.0.1:8000", description: "FastAPI (uvicorn). WS enabled.",          advanced_config: None },
        Template { id: "flask",     name: "Flask",         category: "Python",  default_upstream: "http://127.0.0.1:5000", description: "Flask dev server.",                      advanced_config: None },

        // ── PHP ──
        Template { id: "laravel",   name: "Laravel",       category: "PHP",     default_upstream: "http://127.0.0.1:8000", description: "php artisan serve.",                     advanced_config: None },
        Template { id: "symfony",   name: "Symfony",       category: "PHP",     default_upstream: "http://127.0.0.1:8000", description: "symfony serve.",                          advanced_config: None },
        Template { id: "wordpress", name: "WordPress",     category: "PHP",     default_upstream: "http://127.0.0.1:8080", description: "Local WP / MAMP / XAMPP.",                advanced_config: None },

        // ── Ruby / Other ──
        Template { id: "rails",     name: "Rails",         category: "Ruby",    default_upstream: "http://127.0.0.1:3000", description: "rails server.",                          advanced_config: None },

        // ── Compiled ──
        Template { id: "rust",      name: "Rust (Axum/Actix)", category: "Compiled", default_upstream: "http://127.0.0.1:8080", description: "Rust web server.",                  advanced_config: None },
        Template { id: "go",        name: "Go",            category: "Compiled", default_upstream: "http://127.0.0.1:8080", description: "Go web server.",                         advanced_config: None },
        Template { id: "java",      name: "Spring Boot",   category: "Compiled", default_upstream: "http://127.0.0.1:8080", description: "Spring Boot application.",               advanced_config: None },
        Template { id: "dotnet",    name: ".NET",          category: "Compiled", default_upstream: "http://127.0.0.1:5000", description: "dotnet run default port.",               advanced_config: None },

        // ── Misc ──
        Template { id: "phoenix",   name: "Phoenix",       category: "Misc",    default_upstream: "http://127.0.0.1:4000", description: "Elixir Phoenix + LiveView.",             advanced_config: None },
        Template { id: "ollama",    name: "Ollama",        category: "Misc",    default_upstream: "http://127.0.0.1:11434", description: "Local LLM API.",                        advanced_config: None },
        Template { id: "grafana",   name: "Grafana",       category: "Misc",    default_upstream: "http://127.0.0.1:3000", description: "Grafana dashboard.",                     advanced_config: None },
    ]
}
