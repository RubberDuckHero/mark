use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

const GLOBAL_SCOPE: &str = "__global__";

type Marks = HashMap<String, String>;
type Db = HashMap<String, Marks>;

#[derive(Parser, Debug)]
#[command(name = "mark")]
#[command(
    about = "Mark locations to quickly jump back to, will create at repo level if it can otherwise globally"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    mark: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Command {
    List,
    Add { name: String },
    Rm { name: String },
}

#[derive(Debug, Clone)]
enum Scope {
    Global,
    Repo(PathBuf),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let scope = detect_scope(&cwd)?;

    match cli.command {
        Some(Command::List) => cmd_list(&scope),
        Some(Command::Add { name }) => cmd_add(&scope, &cwd, &name),
        Some(Command::Rm { name }) => cmd_rm(&scope, &name),
        None => {
            let name = cli.mark.ok_or_else(|| anyhow!("Missing mark name"))?;
            cmd_jump(&scope, &name)
        }
    }
}

fn cmd_list(scope: &Scope) -> Result<()> {
    let db = load_db()?;

    match scope {
        Scope::Repo(root) => {
            println!("Repo: {}\n", root.display());

            let repo_key = scope_key(scope);
            let repo_marks = db.get(&repo_key);
            let global_marks = db.get(GLOBAL_SCOPE);

            print_marks("Repo marks", repo_marks, Some(root));
            println!();
            print_marks("Global marks", global_marks, None);
        }
        Scope::Global => {
            print_marks("Global marks", db.get(GLOBAL_SCOPE), None);
        }
    }

    Ok(())
}

fn print_marks(title: &str, marks: Option<&Marks>, repo_root: Option<&Path>) {
    println!("{title}");

    let Some(marks) = marks else {
        println!("  (none)");
        return;
    };

    let mut entries: Vec<_> = marks.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));

    for (name, stored_path) in entries {
        let display_path = match repo_root {
            Some(root) => root.join(stored_path).display().to_string(),
            None => stored_path.clone(),
        };
        println!("  {:<16} {}", name, display_path);
    }
}

fn cmd_add(scope: &Scope, cwd: &Path, name: &str) -> Result<()> {
    let mut db = load_db()?;
    let key = scope_key(scope);

    let stored_path = match scope {
        Scope::Repo(root) => cwd
            .strip_prefix(root)
            .context("Failed to store path relative to repo root")?
            .to_string_lossy()
            .to_string(),
        Scope::Global => cwd
            .canonicalize()
            .context("Failed to canonicalize current directory")?
            .to_string_lossy()
            .to_string(),
    };

    db.entry(key)
        .or_default()
        .insert(name.to_string(), stored_path);
    save_db(&db)
}

fn cmd_rm(scope: &Scope, name: &str) -> Result<()> {
    let mut db = load_db()?;
    let key = scope_key(scope);

    if let Some(marks) = db.get_mut(&key) {
        marks.remove(name);
        if marks.is_empty() {
            db.remove(&key);
        }
    }

    save_db(&db)
}

fn cmd_jump(scope: &Scope, name: &str) -> Result<()> {
    let db = load_db()?;

    match scope {
        Scope::Repo(root) => {
            if let Some(rel) = db.get(&scope_key(scope)).and_then(|m| m.get(name)) {
                println!("{}", root.join(rel).display());
                return Ok(());
            }
            if let Some(abs) = db.get(GLOBAL_SCOPE).and_then(|m| m.get(name)) {
                println!("{abs}");
                return Ok(());
            }

            Err(anyhow!("Mark not found: {name}"))
        }
        Scope::Global => {
            let abs = db
                .get(GLOBAL_SCOPE)
                .and_then(|m| m.get(name))
                .ok_or_else(|| anyhow!("Mark not found: {name}"))?;
            println!("{abs}");
            Ok(())
        }
    }
}

fn detect_scope(cwd: &Path) -> Result<Scope> {
    match find_repo_root(cwd) {
        Some(root) => Ok(Scope::Repo(
            root.canonicalize()
                .context("Failed to canonicalize repo root")?,
        )),
        None => Ok(Scope::Global),
    }
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(path) = current {
        if path.join(".git").exists() {
            return Some(path.to_path_buf());
        }
        current = path.parent();
    }

    None
}

fn scope_key(scope: &Scope) -> String {
    match scope {
        Scope::Global => GLOBAL_SCOPE.to_string(),
        Scope::Repo(root) => root.to_string_lossy().to_string(),
    }
}

fn db_path() -> Result<PathBuf> {
    let dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow!("Could not determine local data directory"))?
        .join("mark");

    fs::create_dir_all(&dir).context("Failed to create data directory")?;
    Ok(dir.join("marks.json"))
}

fn load_db() -> Result<Db> {
    let path = db_path()?;
    if !path.exists() {
        return Ok(HashMap::new());
    }

    let content = fs::read_to_string(&path).context("Failed to read marks database")?;
    let db = serde_json::from_str(&content).context("Failed to parse marks database")?;
    Ok(db)
}

fn save_db(db: &Db) -> Result<()> {
    let path = db_path()?;
    let content = serde_json::to_string_pretty(db).context("Failed to serialize marks database")?;
    fs::write(&path, content).context("Failed to write marks database")?;
    Ok(())
}
