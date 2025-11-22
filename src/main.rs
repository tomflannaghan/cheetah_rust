use clap::Parser;
use regex::RegexBuilder;
use rusqlite::Connection;
use std::{path::PathBuf, process};

#[derive(Parser)]
#[command(author, version, about = "Search words in a SQLite DB with a regex")]
struct Cli {
    /// Path to sqlite file
    db: PathBuf,

    /// Regular expression to match
    pattern: String,

    /// Case-insensitive match
    #[arg(short, long)]
    ignore_case: bool,
}

fn main() {
    let cli = Cli::parse();

    let re = RegexBuilder::new(&cli.pattern)
        .case_insensitive(cli.ignore_case)
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Invalid regex: {}", e);
            process::exit(2);
        });

    let conn = Connection::open(&cli.db).unwrap_or_else(|e| {
        eprintln!("Failed to open `{}`: {}", cli.db.display(), e);
        process::exit(3);
    });

    let mut stmt = conn
        .prepare("SELECT word, canonical_form FROM word")
        .unwrap_or_else(|e| {
        eprintln!("Failed to prepare query: {}", e);
        process::exit(4);
    });

    let rows = stmt
        .query_map([], |row| {
            // column 0 -> word, column 1 -> canonical_form
            let word = row.get::<_, String>(0)?;
            let canonical = row.get::<_, String>(1)?;
            Ok((word, canonical))
        })
        .unwrap_or_else(|e| {
            eprintln!("Query failed: {}", e);
            process::exit(5);
        });
    for row in rows {
        match row {
            Ok((word, canonical)) if re.is_match(&canonical) => println!("{}", word),
            Ok(_) => {}
            Err(e) => eprintln!("Row read error: {}", e),
        }
    }
}
