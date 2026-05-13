use std::path::Path;

const SKILL_CONTENT: &str = include_str!("../../.claude/commands/hyh.md");

const BEGIN_MARKER: &str = "<!-- HyperHost BEGIN -->";
const END_MARKER: &str = "<!-- HyperHost END -->";

pub fn skill_content_raw() -> &'static str {
    SKILL_CONTENT
}

fn strip_frontmatter(content: &str) -> &str {
    if !content.starts_with("---") {
        return content;
    }
    if let Some(end) = content[3..].find("---") {
        let after = 3 + end + 3;
        content[after..].trim_start_matches(['\r', '\n'])
    } else {
        content
    }
}

fn upsert_marked_section(existing: &str, body: &str) -> String {
    let section = format!("{}\n{}\n{}\n", BEGIN_MARKER, body.trim(), END_MARKER);

    let cleaned = if let Some(start) = existing.find(BEGIN_MARKER) {
        if let Some(end) = existing[start..].find(END_MARKER) {
            let before = &existing[..start];
            let after = &existing[start + end + END_MARKER.len()..];
            format!("{}{}", before.trim_end(), after)
        } else {
            existing[..start].trim_end().to_string()
        }
    } else {
        existing.to_string()
    };

    if cleaned.trim().is_empty() {
        section
    } else {
        format!("{}\n\n{}", cleaned.trim_end(), section)
    }
}

pub fn install_instructions_file(file_path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let body = strip_frontmatter(SKILL_CONTENT);

    let existing = if file_path.exists() {
        std::fs::read_to_string(file_path)?
    } else {
        String::new()
    };

    let final_content = upsert_marked_section(&existing, body);
    std::fs::write(file_path, final_content)?;
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct SkillInstallResult {
    pub target: String,
    pub ok: bool,
    pub path: String,
}

pub fn install_project_skills(project_path: &Path) -> Vec<SkillInstallResult> {
    let body = strip_frontmatter(SKILL_CONTENT);
    let mut results = Vec::new();

    // Claude Code — .claude/commands/hyh.md (with frontmatter)
    let claude_path = project_path.join(".claude").join("commands").join("hyh.md");
    let claude_ok = (|| -> anyhow::Result<()> {
        if let Some(parent) = claude_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&claude_path, SKILL_CONTENT)?;
        Ok(())
    })();
    results.push(SkillInstallResult {
        target: "Claude Code (.claude/commands/hyh.md)".into(),
        ok: claude_ok.is_ok(),
        path: claude_path.display().to_string(),
    });

    // Gemini CLI — GEMINI.md
    let gemini_path = project_path.join("GEMINI.md");
    let gemini_ok = install_marked_file(&gemini_path, body);
    results.push(SkillInstallResult {
        target: "Gemini CLI (GEMINI.md)".into(),
        ok: gemini_ok.is_ok(),
        path: gemini_path.display().to_string(),
    });

    // Codex CLI — AGENTS.md (project-level)
    let codex_path = project_path.join("AGENTS.md");
    let codex_ok = install_marked_file(&codex_path, body);
    results.push(SkillInstallResult {
        target: "Codex CLI (AGENTS.md)".into(),
        ok: codex_ok.is_ok(),
        path: codex_path.display().to_string(),
    });

    // Claude Code — CLAUDE.md (project-level context)
    let claude_md_path = project_path.join("CLAUDE.md");
    let claude_md_ok = install_marked_file(&claude_md_path, body);
    results.push(SkillInstallResult {
        target: "Claude Code (CLAUDE.md)".into(),
        ok: claude_md_ok.is_ok(),
        path: claude_md_path.display().to_string(),
    });

    results
}

fn install_marked_file(file_path: &Path, body: &str) -> anyhow::Result<()> {
    let existing = if file_path.exists() {
        std::fs::read_to_string(file_path)?
    } else {
        String::new()
    };

    let final_content = upsert_marked_section(&existing, body);
    std::fs::write(file_path, final_content)?;
    Ok(())
}
