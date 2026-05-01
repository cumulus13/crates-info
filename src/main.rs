use clap::{Parser, Subcommand};
use colored::*;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use termimad::crossterm::style::Color::*;
use termimad::MadSkin;
use terminal_size::{terminal_size, Width};
use textwrap::wrap;

// ─────────────────────────────────────────────────────────────
// CLI definition
// ─────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[command(
    name = "cratesinfo",
    bin_name = "cratesinfo",
    author = "Hadi Cahyadi <cumulus13@gmail.com>",
    version,
    about = "Get detailed info about Rust crates from crates.io",
    long_about = "A fast CLI tool to inspect Rust crates – metadata, versions, dependencies, features and README.",
    after_help = "EXAMPLES:\n  cratesinfo info serde\n  cratesinfo versions tokio\n  cratesinfo deps serde 1.0.195\n  cratesinfo search async runtime\n  cratesinfo readme actix-web"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show full details of a crate (metadata + latest version info)
    Info {
        /// Crate name
        crate_name: String,
        /// Show README/description as well
        #[arg(short = 'r', long)]
        readme: bool,
    },
    /// List all published versions of a crate
    Versions {
        /// Crate name
        crate_name: String,
        /// Show all versions (not just last 20)
        #[arg(short, long)]
        all: bool,
    },
    /// List dependencies of a specific crate version
    Deps {
        /// Crate name
        crate_name: String,
        /// Version (default: latest)
        version: Option<String>,
    },
    /// Show the README / description of a crate
    Readme {
        /// Crate name
        crate_name: String,
        /// Version (default: latest)
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Search crates.io
    Search {
        /// Search terms
        #[arg(required = true, num_args = 1..)]
        query: Vec<String>,
        /// Number of results (default: 10)
        #[arg(short, long, default_value = "10")]
        limit: u32,
    },
    /// Show owners of a crate
    Owners {
        /// Crate name
        crate_name: String,
    },
}

// ─────────────────────────────────────────────────────────────
// API response structures
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
struct CrateResponse {
    #[serde(rename = "crate")]
    krate: CrateInfo,
    versions: Option<Vec<VersionInfo>>,
    categories: Option<Vec<Category>>,
    keywords: Option<Vec<Keyword>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CrateInfo {
    id: String,
    name: String,
    description: Option<String>,
    downloads: u64,
    recent_downloads: Option<u64>,
    max_version: String,
    newest_version: Option<String>,
    documentation: Option<String>,
    homepage: Option<String>,
    repository: Option<String>,
    created_at: String,
    updated_at: String,
    max_stable_version: Option<String>,
    exact_match: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct VersionInfo {
    id: u64,
    #[serde(rename = "crate")]
    krate: String,
    num: String,
    dl_path: Option<String>,
    downloads: u64,
    features: Option<serde_json::Value>,
    yanked: bool,
    license: Option<String>,
    published_by: Option<PublishedBy>,
    created_at: String,
    updated_at: String,
    rust_version: Option<String>,
    crate_size: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PublishedBy {
    login: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Category {
    category: String,
    slug: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Keyword {
    keyword: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct DepsResponse {
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Dependency {
    id: u64,
    version_id: u64,
    crate_id: String,
    req: String,
    optional: bool,
    default_features: bool,
    features: Vec<String>,
    kind: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct SearchResponse {
    crates: Vec<SearchCrate>,
    meta: SearchMeta,
}

#[derive(Debug, Deserialize, Serialize)]
struct SearchCrate {
    id: String,
    name: String,
    description: Option<String>,
    downloads: u64,
    recent_downloads: Option<u64>,
    max_version: String,
    updated_at: String,
    exact_match: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SearchMeta {
    total: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct OwnersResponse {
    users: Vec<Owner>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Owner {
    id: u64,
    login: String,
    name: Option<String>,
    avatar: Option<String>,
    url: Option<String>,
    kind: Option<String>,
}

// ─────────────────────────────────────────────────────────────
// HTTP client wrapper
// ─────────────────────────────────────────────────────────────

struct CratesClient {
    client: Client,
    base: String,
}

impl CratesClient {
    fn new() -> Self {
        let client = Client::builder()
            .user_agent(
                "cratesinfo/0.1.0 (https://github.com/cumulus13/crates-info; cumulus13@gmail.com)",
            )
            .timeout(Duration::from_secs(15))
            .build()
            .expect("Failed to build HTTP client");
        CratesClient {
            client,
            base: "https://crates.io/api/v1".to_string(),
        }
    }

    fn get_crate(&self, name: &str) -> Result<CrateResponse, String> {
        let url = format!("{}/crates/{}", self.base, name);
        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Network error: {e}"))?
            .json::<CrateResponse>()
            .map_err(|e| format!("Parse error: {e}"))
    }

    fn get_versions(&self, name: &str) -> Result<Vec<VersionInfo>, String> {
        #[derive(Deserialize)]
        struct Wrap {
            versions: Vec<VersionInfo>,
        }
        let url = format!("{}/crates/{}/versions", self.base, name);
        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Network error: {e}"))?
            .json::<Wrap>()
            .map(|w| w.versions)
            .map_err(|e| format!("Parse error: {e}"))
    }

    fn get_deps(&self, name: &str, version: &str) -> Result<DepsResponse, String> {
        let url = format!("{}/crates/{}/{}/dependencies", self.base, name, version);
        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Network error: {e}"))?
            .json::<DepsResponse>()
            .map_err(|e| format!("Parse error: {e}"))
    }

    fn get_readme(&self, name: &str, version: &str) -> Result<String, String> {
        let url = format!("{}/crates/{}/{}/readme", self.base, name, version);
        let resp = self
            .client
            .get(&url)
            .send()
            .map_err(|e| format!("Network error: {e}"))?;
        if resp.status().is_success() {
            resp.text().map_err(|e| format!("Read error: {e}"))
        } else {
            Err(format!("No README available (HTTP {})", resp.status()))
        }
    }

    fn search(&self, query: &str, limit: u32) -> Result<SearchResponse, String> {
        let url = format!("{}/crates?q={}&per_page={}", self.base, query, limit);
        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Network error: {e}"))?
            .json::<SearchResponse>()
            .map_err(|e| format!("Parse error: {e}"))
    }

    fn get_owners(&self, name: &str) -> Result<OwnersResponse, String> {
        let url = format!("{}/crates/{}/owners", self.base, name);
        self.client
            .get(&url)
            .send()
            .map_err(|e| format!("Network error: {e}"))?
            .json::<OwnersResponse>()
            .map_err(|e| format!("Parse error: {e}"))
    }
}

// ─────────────────────────────────────────────────────────────
// Display helpers
// ─────────────────────────────────────────────────────────────

fn term_width() -> usize {
    terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(100)
        .min(120)
}

fn separator(ch: char) {
    let w = term_width();
    println!("{}", ch.to_string().repeat(w).dimmed());
}

fn header(title: &str) {
    let w = term_width();
    println!();
    println!("{}", "═".repeat(w).bright_cyan());
    let pad = (w.saturating_sub(title.len())) / 2;
    println!("{}{}", " ".repeat(pad), title.bright_white().bold());
    println!("{}", "═".repeat(w).bright_cyan());
}

fn field(label: &str, value: &str) {
    println!(
        "  {:<22} {}",
        format!("{}:", label).cyan().bold(),
        value.bright_white()
    );
}

fn fmt_num(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn fmt_date(s: &str) -> String {
    s.replace('T', " ")
        .split('.')
        .next()
        .unwrap_or(s)
        .trim_end_matches('Z')
        .to_string()
        + " UTC"
}

fn fmt_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn strip_html(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default_dark();

    // H1 — bright magenta bold, underlined
    skin.headers[0].set_fg(Rgb {
        r: 255,
        g: 100,
        b: 200,
    });
    skin.headers[0].add_attr(termimad::crossterm::style::Attribute::Bold);
    skin.headers[0].add_attr(termimad::crossterm::style::Attribute::Underlined);

    // H2 — bright cyan bold
    skin.headers[1].set_fg(Rgb {
        r: 80,
        g: 220,
        b: 255,
    });
    skin.headers[1].add_attr(termimad::crossterm::style::Attribute::Bold);

    // H3 — yellow bold
    skin.headers[2].set_fg(Rgb {
        r: 255,
        g: 200,
        b: 60,
    });
    skin.headers[2].add_attr(termimad::crossterm::style::Attribute::Bold);

    // Bold — bright green
    skin.bold.set_fg(Rgb {
        r: 120,
        g: 255,
        b: 120,
    });
    skin.bold
        .add_attr(termimad::crossterm::style::Attribute::Bold);

    // Italic — light orange
    skin.italic.set_fg(Rgb {
        r: 255,
        g: 180,
        b: 80,
    });

    // Inline code — dark bg, bright text
    skin.inline_code.set_fg(Rgb {
        r: 255,
        g: 130,
        b: 80,
    });
    skin.inline_code.set_bg(Rgb {
        r: 40,
        g: 40,
        b: 50,
    });

    // Code blocks
    skin.code_block.set_fg(Rgb {
        r: 200,
        g: 200,
        b: 200,
    });
    skin.code_block.set_bg(Rgb {
        r: 30,
        g: 30,
        b: 40,
    });

    // Bullet — bright cyan bullet char
    skin.bullet = termimad::StyledChar::from_fg_char(
        Rgb {
            r: 80,
            g: 220,
            b: 255,
        },
        '❯',
    );

    // Quotes — purple italic
    skin.quote_mark = termimad::StyledChar::from_fg_char(
        Rgb {
            r: 180,
            g: 100,
            b: 255,
        },
        '▌',
    );

    // Horizontal rule
    skin.horizontal_rule = termimad::StyledChar::from_fg_char(
        Rgb {
            r: 60,
            g: 80,
            b: 100,
        },
        '─',
    );

    // Table
    skin.table.set_fg(Rgb {
        r: 80,
        g: 140,
        b: 200,
    });

    // Strikeout
    skin.strikeout.set_fg(Rgb {
        r: 120,
        g: 120,
        b: 120,
    });

    skin
}

fn render_text_readme(raw: &str) {
    let is_html = raw.trim_start().starts_with('<') || raw.contains("</");
    let text = if is_html {
        strip_html(raw)
    } else {
        raw.to_string()
    };

    let skin = make_skin();
    let width = term_width() as u16;

    // termimad wraps to width; print_text handles full markdown blocks
    let area = termimad::Area::new(2, 0, width.saturating_sub(4), 9999);
    // if let Err(_) = skin.write_in_area(&text, &area) {
    if skin.write_in_area(&text, &area).is_err() {
        // fallback: plain print_text (no area positioning)
        skin.print_text(&text);
    }
}

// ─────────────────────────────────────────────────────────────
// Command handlers
// ─────────────────────────────────────────────────────────────

fn cmd_info(client: &CratesClient, name: &str, show_readme: bool) {
    print!(
        "  {} {}/{}  ",
        "Fetching".dimmed(),
        "crates.io".bright_cyan(),
        name.bright_yellow()
    );
    let resp = match client.get_crate(name) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", "✗ failed".red());
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };
    println!("{}", "✓".green().bold());

    let krate = &resp.krate;
    header(&format!("  {}  ", krate.name.to_uppercase()));

    if let Some(desc) = &krate.description {
        let width = term_width() - 6;
        println!();
        for line in wrap(desc.trim(), width) {
            println!("  {}", line.bright_white());
        }
        println!();
    }

    separator('─');
    println!("  📦 {}", "METADATA".bright_cyan().bold());
    separator('─');

    field("📛  Name", &krate.name);
    field(
        "🏷️   Latest Version",
        &krate.max_version.bright_green().to_string(),
    );
    if let Some(stable) = &krate.max_stable_version {
        if stable != &krate.max_version {
            field("✅  Latest Stable", stable);
        }
    }
    field(
        "⬇️   Downloads (total)",
        &fmt_num(krate.downloads).yellow().to_string(),
    );
    if let Some(r) = krate.recent_downloads {
        field("📈  Downloads (90d)", &fmt_num(r).yellow().to_string());
    }
    field(
        "🗓️   Created",
        &fmt_date(&krate.created_at).dimmed().to_string(),
    );
    field(
        "🔄  Updated",
        &fmt_date(&krate.updated_at).dimmed().to_string(),
    );

    println!();
    println!("  🔗 {}", "LINKS".bright_cyan().bold());
    separator('─');
    if let Some(v) = &krate.documentation {
        field("📚  Docs", v);
    }
    if let Some(v) = &krate.homepage {
        field("🏠  Homepage", v);
    }
    if let Some(v) = &krate.repository {
        field("💻  Repository", v);
    }

    if let Some(keywords) = &resp.keywords {
        if !keywords.is_empty() {
            println!();
            separator('─');
            println!("  🏷️  {}", "KEYWORDS".bright_cyan().bold());
            separator('─');
            let kw: Vec<String> = keywords
                .iter()
                .map(|k| {
                    format!(" {} ", k.keyword)
                        .on_bright_black()
                        .bright_green()
                        .to_string()
                })
                .collect();
            println!("  {}", kw.join("  "));
        }
    }

    if let Some(cats) = &resp.categories {
        if !cats.is_empty() {
            println!();
            separator('─');
            println!("  📂 {}", "CATEGORIES".bright_cyan().bold());
            separator('─');
            for c in cats {
                println!("  {} {}", "❯".yellow(), c.category.bright_white());
            }
        }
    }

    if let Some(versions) = &resp.versions {
        if let Some(latest) = versions.first() {
            println!();
            separator('─');
            println!("  🔖 {}", "LATEST VERSION DETAILS".bright_cyan().bold());
            separator('─');
            field("🔢  Version", &latest.num.bright_green().to_string());
            field("⚖️   License", latest.license.as_deref().unwrap_or("N/A"));
            if let Some(size) = latest.crate_size {
                field("📦  Crate Size", &fmt_size(size));
            }
            if let Some(rv) = &latest.rust_version {
                field("🦀  MSRV", rv);
            }
            field(
                "⬇️   Downloads",
                &fmt_num(latest.downloads).yellow().to_string(),
            );
            let yanked_display = if latest.yanked {
                "⛔ Yes".red().to_string()
            } else {
                "✅ No".green().to_string()
            };
            field("🚫  Yanked", &yanked_display);
            if let Some(pb) = &latest.published_by {
                let pub_str = match &pb.name {
                    Some(n) => format!("{} (@{})", n, pb.login),
                    None => format!("@{}", pb.login),
                };
                field("👤  Published By", &pub_str);
            }
            field(
                "📅  Released",
                &fmt_date(&latest.created_at).dimmed().to_string(),
            );

            if let Some(feats) = &latest.features {
                if let Some(obj) = feats.as_object() {
                    if !obj.is_empty() {
                        println!();
                        separator('─');
                        println!("  ⚙️  {}", "FEATURES".bright_cyan().bold());
                        separator('─');
                        for (feat, deps) in obj {
                            let dep_strs: Vec<String> = deps
                                .as_array()
                                .map(|a| {
                                    a.iter()
                                        .filter_map(|v| v.as_str())
                                        .map(|s| s.dimmed().to_string())
                                        .collect()
                                })
                                .unwrap_or_default();
                            if dep_strs.is_empty() {
                                println!("  {} {}", "✔".green(), feat.bright_white());
                            } else {
                                println!(
                                    "  {} {} {}",
                                    "✔".green(),
                                    feat.bright_white(),
                                    format!("({})", dep_strs.join(", ")).dimmed()
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    println!();
    separator('═');
    println!(
        "  {} {}  {}",
        "🚀 Add to project:".dimmed(),
        format!("cargo add {}", krate.name).bright_yellow().bold(),
        format!("# v{}", krate.max_version).dimmed()
    );
    separator('═');

    if show_readme {
        cmd_readme(client, name, &Some(krate.max_version.clone()));
    }
}

fn cmd_versions(client: &CratesClient, name: &str, show_all: bool) {
    print!(
        "  {} {} versions... ",
        "Fetching".dimmed(),
        name.bright_yellow()
    );
    let versions = match client.get_versions(name) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", "✗ failed".red());
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };
    println!("{}", "✓".green().bold());

    let display: Vec<&VersionInfo> = if show_all {
        versions.iter().collect()
    } else {
        versions.iter().take(20).collect()
    };

    header(&format!(
        "  📋 {} — {} versions  ",
        name.to_uppercase(),
        versions.len()
    ));

    println!(
        "  {:<18} {:<24} {:<14} {:<12} {:<18} {}",
        "VERSION".bright_cyan().bold(),
        "RELEASED".bright_cyan().bold(),
        "DOWNLOADS".bright_cyan().bold(),
        "SIZE".bright_cyan().bold(),
        "LICENSE".bright_cyan().bold(),
        "FLAGS".bright_cyan().bold()
    );
    separator('─');

    for v in &display {
        let yanked_str = if v.yanked {
            " [yanked]".red().to_string()
        } else {
            String::new()
        };
        let size_str = v
            .crate_size
            .map(fmt_size)
            .unwrap_or_else(|| "—".to_string());
        let lic_str = v.license.clone().unwrap_or_else(|| "—".to_string());

        println!(
            "  {:<18} {:<24} {:<14} {:<12} {:<18}{}",
            v.num.bright_green(),
            fmt_date(&v.created_at).dimmed(),
            fmt_num(v.downloads).yellow(),
            size_str,
            lic_str,
            yanked_str
        );
    }

    if !show_all && versions.len() > 20 {
        println!();
        println!(
            "  {} {} more — use {} to see all",
            "…".dimmed(),
            (versions.len() - 20).to_string().yellow(),
            "--all".bright_cyan()
        );
    }
    separator('═');
}

fn cmd_deps(client: &CratesClient, name: &str, version: Option<String>) {
    let ver = match version {
        Some(v) => v,
        None => {
            print!("  {} latest version... ", "Resolving".dimmed());
            match client.get_crate(name) {
                Ok(r) => {
                    let v = r.krate.max_version.clone();
                    println!("{} ({})", "✓".green().bold(), v.bright_green());
                    v
                }
                Err(e) => {
                    println!("{}", "✗".red());
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }
    };

    print!(
        "  {} {} {} deps... ",
        "Fetching".dimmed(),
        name.bright_yellow(),
        ver.bright_green()
    );
    let deps_resp = match client.get_deps(name, &ver) {
        Ok(d) => d,
        Err(e) => {
            println!("{}", "✗ failed".red());
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };
    println!("{}", "✓".green().bold());

    header(&format!(
        "  {} v{} — Dependencies  ",
        name.to_uppercase(),
        ver
    ));

    let normal: Vec<&Dependency> = deps_resp
        .dependencies
        .iter()
        .filter(|d| d.kind == "normal")
        .collect();
    let dev: Vec<&Dependency> = deps_resp
        .dependencies
        .iter()
        .filter(|d| d.kind == "dev")
        .collect();
    let build: Vec<&Dependency> = deps_resp
        .dependencies
        .iter()
        .filter(|d| d.kind == "build")
        .collect();

    let print_deps = |label: &str, color_fn: fn(&str) -> ColoredString, deps: &[&Dependency]| {
        if deps.is_empty() {
            return;
        }
        println!(
            "  {} ({}):",
            color_fn(label).bold(),
            deps.len().to_string().yellow()
        );
        separator('─');
        for d in deps {
            let optional = if d.optional {
                " [optional]".dimmed().to_string()
            } else {
                String::new()
            };
            let feats = if !d.features.is_empty() {
                format!(" features=[{}]", d.features.join(", "))
                    .dimmed()
                    .to_string()
            } else {
                String::new()
            };
            println!(
                "  {:<34} {:<16}{}{}",
                d.crate_id.bright_white(),
                d.req.green(),
                optional,
                feats
            );
        }
        println!();
    };

    print_deps("🧩 NORMAL DEPENDENCIES", |s| s.bright_cyan(), &normal);
    print_deps("🧪 DEV DEPENDENCIES", |s| s.yellow(), &dev);
    print_deps("🔨 BUILD DEPENDENCIES", |s| s.magenta(), &build);

    if deps_resp.dependencies.is_empty() {
        println!(
            "  {} This crate has no external dependencies.",
            "🎉".bright_cyan()
        );
        println!();
    }
    separator('═');
}

fn cmd_readme(client: &CratesClient, name: &str, version: &Option<String>) {
    let ver = match version {
        Some(v) => v.clone(),
        None => {
            print!("  {} latest version... ", "Resolving".dimmed());
            match client.get_crate(name) {
                Ok(r) => {
                    let v = r.krate.max_version.clone();
                    println!("{} ({})", "✓".green().bold(), v.bright_green());
                    v
                }
                Err(e) => {
                    println!("{}", "✗".red());
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
        }
    };

    print!(
        "  {} README for {} {}... ",
        "Fetching".dimmed(),
        name.bright_yellow(),
        ver.bright_green()
    );
    let raw = match client.get_readme(name, &ver) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", "✗".red());
            println!("  {} {}", "ℹ".bright_cyan(), e.dimmed());
            return;
        }
    };
    println!("{}", "✓".green().bold());

    header(&format!("  {} v{} — README  ", name.to_uppercase(), ver));
    println!();
    render_text_readme(&raw);
    println!();
    separator('═');
}

fn cmd_search(client: &CratesClient, query: &[String], limit: u32) {
    let q = query.join(" ");
    print!("  {} \"{}\"... ", "Searching".dimmed(), q.bright_yellow());
    let results = match client.search(&q, limit) {
        Ok(r) => r,
        Err(e) => {
            println!("{}", "✗ failed".red());
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };
    println!("{}", "✓".green().bold());

    header(&format!(
        "  🔍 Search: \"{}\" — {} total results  ",
        q, results.meta.total
    ));

    if results.crates.is_empty() {
        println!("  {} No results found.", "ℹ".bright_cyan());
        separator('═');
        return;
    }

    let width = term_width();

    for (i, c) in results.crates.iter().enumerate() {
        let match_tag = if c.exact_match == Some(true) {
            format!(" {}", "[exact match]".bright_green())
        } else {
            String::new()
        };
        println!(
            "  {} 📦 {}{} {}",
            format!("{:>2}.", i + 1).dimmed(),
            c.name.bright_yellow().bold(),
            match_tag,
            format!("v{}", c.max_version).bright_green()
        );

        if let Some(desc) = &c.description {
            for line in wrap(desc.trim(), width - 8) {
                println!("      {}", line.white());
            }
        }

        let dl = fmt_num(c.downloads);
        let recent = c
            .recent_downloads
            .map(|r| format!("  recent: {}", fmt_num(r)))
            .unwrap_or_default();
        println!(
            "      {} {}{}  {} {}",
            "⬇️".cyan(),
            dl.yellow(),
            recent.dimmed(),
            "updated:".dimmed(),
            fmt_date(&c.updated_at).dimmed()
        );
        println!("      🚀 {}", format!("cargo add {}", c.name).dimmed());

        if i < results.crates.len() - 1 {
            separator('─');
        }
    }
    separator('═');
}

fn cmd_owners(client: &CratesClient, name: &str) {
    print!(
        "  {} owners of {}... ",
        "Fetching".dimmed(),
        name.bright_yellow()
    );
    let owners = match client.get_owners(name) {
        Ok(o) => o,
        Err(e) => {
            println!("{}", "✗ failed".red());
            eprintln!("{} {}", "Error:".red().bold(), e);
            std::process::exit(1);
        }
    };
    println!("{}", "✓".green().bold());

    header(&format!("  👥 {} — Owners  ", name.to_uppercase()));

    for owner in &owners.users {
        let kind = owner.kind.as_deref().unwrap_or("user");
        let display_name = match &owner.name {
            Some(n) if !n.is_empty() => format!("{} (@{})", n, owner.login),
            _ => format!("@{}", owner.login),
        };
        let icon = if kind == "team" { "🏢" } else { "👤" };
        println!("  {} {}", icon, display_name.bright_white().bold());
        if let Some(url) = &owner.url {
            println!("      {} {}", "URL:".dimmed(), url.cyan());
        }
        println!(
            "      {} {} / id: {}",
            "Kind:".dimmed(),
            kind.yellow(),
            owner.id.to_string().dimmed()
        );
        println!();
    }
    separator('═');
}

// ─────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let client = CratesClient::new();

    println!();
    println!(
        "  {} {} {} {}",
        "🦀 cratesinfo".bright_cyan().bold(),
        "─".dimmed(),
        "crates.io".bright_yellow(),
        "explorer 📦".dimmed()
    );
    println!();

    match cli.command {
        Commands::Info { crate_name, readme } => {
            cmd_info(&client, &crate_name, readme);
        }
        Commands::Versions { crate_name, all } => {
            cmd_versions(&client, &crate_name, all);
        }
        Commands::Deps {
            crate_name,
            version,
        } => {
            cmd_deps(&client, &crate_name, version);
        }
        Commands::Readme {
            crate_name,
            version,
        } => {
            cmd_readme(&client, &crate_name, &version);
        }
        Commands::Search { query, limit } => {
            cmd_search(&client, &query, limit);
        }
        Commands::Owners { crate_name } => {
            cmd_owners(&client, &crate_name);
        }
    }
}
