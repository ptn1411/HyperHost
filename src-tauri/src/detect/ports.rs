use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

const COMMON_PORTS: &[u16] = &[
    1337, 2368, 3000, 3001, 3002, 3003, 3030, 3333, 4000, 4200, 4321,
    5000, 5001, 5173, 5174, 5175, 5555, 6006, 7000, 7001, 7777,
    8000, 8001, 8080, 8081, 8082, 8088, 8090, 8443, 8787, 8888,
    9000, 9001, 9090, 9229, 10000, 11434, 19006,
];

pub async fn scan_listening_ports() -> Vec<u16> {
    let mut handles = Vec::with_capacity(COMMON_PORTS.len());
    for &port in COMMON_PORTS {
        handles.push(tokio::spawn(async move {
            match timeout(
                Duration::from_millis(250),
                TcpStream::connect(("127.0.0.1", port)),
            )
            .await
            {
                Ok(Ok(_)) => Some(port),
                _ => None,
            }
        }));
    }

    let mut open = Vec::new();
    for h in handles {
        if let Ok(Some(p)) = h.await {
            open.push(p);
        }
    }
    open.sort();
    open
}

pub fn guess_framework(port: u16) -> Option<&'static str> {
    match port {
        1337 => Some("Strapi"),
        2368 => Some("Ghost"),
        3000 => Some("Next.js / Node / Rails"),
        3001 | 3002 | 3003 => Some("Node (alt)"),
        3030 => Some("Feathers / Meteor"),
        3333 => Some("NestJS (default)"),
        4000 => Some("Phoenix / Apollo"),
        4200 => Some("Angular CLI"),
        4321 => Some("Astro"),
        5000 => Some("Flask / .NET"),
        5173 | 5174 | 5175 => Some("Vite / SvelteKit"),
        5555 => Some("Prisma Studio"),
        6006 => Some("Storybook"),
        7000 | 7001 => Some("Cassandra / dev"),
        7777 => Some("dev server"),
        8000 => Some("Django / Laravel / FastAPI"),
        8001 => Some("Django (alt)"),
        8080 => Some("Go / Spring Boot / generic"),
        8081 | 8082 => Some("Go / generic (alt)"),
        8088 | 8090 => Some("dev server"),
        8443 => Some("HTTPS dev"),
        8787 => Some("R Shiny"),
        8888 => Some("Jupyter / dev"),
        9000 => Some("PHP-FPM / SonarQube"),
        9001 => Some("Supervisor / dev"),
        9090 => Some("Prometheus / dev"),
        9229 => Some("Node Inspector"),
        10000 => Some("Webmin / dev"),
        11434 => Some("Ollama"),
        19006 => Some("Expo web"),
        _ => None,
    }
}
