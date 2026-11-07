use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use walkdir::WalkDir;
use chrono::Local;

#[derive(Parser)]
#[command(name = "file-integrity", about = "File Integrity Monitor")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create initial baseline
    Init {
        /// Directories to monitor
        dirs: Vec<PathBuf>,
        /// Output baseline file
        #[arg(short, long, default_value = "fim_baseline.json")]
        output: PathBuf,
    },
    /// Check against baseline
    Check {
        /// Directories to check
        dirs: Vec<PathBuf>,
        /// Baseline file
        #[arg(short, long, default_value = "fim_baseline.json")]
        baseline: PathBuf,
    },
}

#[derive(Serialize, Deserialize, Clone)]
struct FileEntry {
    hash: String,
    size: u64,
    recorded: String,
}

fn hash_file(path: &PathBuf) -> Option<(String, u64)> {
    let mut file = fs::File::open(path).ok()?;
    let meta = file.metadata().ok()?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 { break; }
        hasher.update(&buf[..n]);
    }
    Some((hex::encode(hasher.finalize()), meta.len()))
}

fn build_baseline(dirs: &[PathBuf]) -> HashMap<String, FileEntry> {
    let mut map = HashMap::new();
    let now = Local::now().to_rfc3339();
    for dir in dirs {
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                let path = entry.path().to_path_buf();
                if let Some((hash, size)) = hash_file(&path) {
                    map.insert(path.display().to_string(), FileEntry {
                        hash,
                        size,
                        recorded: now.clone(),
                    });
                }
            }
        }
    }
    map
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { dirs, output } => {
            let bl = build_baseline(&dirs);
            let json = serde_json::to_string_pretty(&bl).unwrap();
            fs::write(&output, json).unwrap();
            println!("[+] Baseline created: {} files -> {}", bl.len(), output.display());
        }
        Commands::Check { dirs, baseline } => {
            let old_json = fs::read_to_string(&baseline).unwrap();
            let old: HashMap<String, FileEntry> = serde_json::from_str(&old_json).unwrap();
            let new = build_baseline(&dirs);

            let old_keys: std::collections::HashSet<_> = old.keys().collect();
            let new_keys: std::collections::HashSet<_> = new.keys().collect();

            let added: Vec<_> = new_keys.difference(&old_keys).collect();
            let removed: Vec<_> = old_keys.difference(&new_keys).collect();
            let mut modified = Vec::new();
            for key in old_keys.intersection(&new_keys) {
                if old[*key].hash != new[*key].hash {
                    modified.push(*key);
                }
            }

            println!("\n{}", "=".repeat(50));
            println!("  FILE INTEGRITY CHECK");
            println!("  {}", Local::now().to_rfc3339());
            println!("{}", "=".repeat(50));

            if !added.is_empty() {
                println!("\n  [NEW] ({})", added.len());
                for f in &added { println!("    + {}", f); }
            }
            if !removed.is_empty() {
                println!("\n  [REMOVED] ({})", removed.len());
                for f in &removed { println!("    - {}", f); }
            }
            if !modified.is_empty() {
                println!("\n  [MODIFIED] ({})", modified.len());
                for f in &modified { println!("    ~ {}", f); }
            }
            if added.is_empty() && removed.is_empty() && modified.is_empty() {
                println!("\n  All files intact.");
            } else {
                std::process::exit(1);
            }
        }
    }
}
