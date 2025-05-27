use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use json_diff_checker::json_diff::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "json-diff-checker")]
#[command(author = "Your Name")]
#[command(version = "1.0")]
#[command(about = "Recursively check JSON differences", long_about = None)]
struct Args {
    /// Base JSON file to compare against
    base_file: PathBuf,

    /// JSON files to compare with the base file
    compare_files: Vec<PathBuf>,

    /// Check values as well as structure
    #[arg(short = 'v', long)]
    check_values: bool,

    /// Only check types, ignore value differences (requires -v)
    #[arg(short = 't', long, requires = "check_values")]
    type_only: bool,

    /// Show only summary
    #[arg(short = 's', long)]
    summary: bool,

    /// Export results to JSON file
    #[arg(short = 'e', long)]
    export: Option<PathBuf>,

    /// Include parent paths in missing items
    #[arg(short = 'p', long)]
    include_parents: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComparisonResult {
    base_file: String,
    compare_file: String,
    missing_paths: Vec<String>,
    different_values: Vec<ValueDifference>,
    type_mismatches: Vec<TypeMismatch>,
    statistics: Statistics,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValueDifference {
    path: String,
    base_value: Value,
    compare_value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct Statistics {
    total_paths_checked: usize,
    missing_count: usize,
    different_count: usize,
    type_mismatch_count: usize,
    match_count: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut all_results = Vec::new();

    // load base JSON file
    let base_json = load_json(&args.base_file)?;
    let base_items = get_all_items(&base_json, String::new());

    // print header information
    print_header(&args, &base_items);

    // check each compare file
    for compare_file in &args.compare_files {
        let result = compare_single_file(&args, &base_items, compare_file)?;

        // output results
        if args.summary {
            print_summary(&result);
        } else {
            print_detailed_results(&result, &args);
        }

        all_results.push(result);
    }

    // print overall summary if multiple files are compared
    if args.compare_files.len() > 1 {
        print_overall_summary(&all_results);
    }

    // export results if specified
    if let Some(export_path) = &args.export {
        export_results(export_path, &all_results)?;
        println!(
            "\n{}",
            format!("✓ Results exported to {:?}", export_path)
                .green()
                .bold()
        );
    }

    Ok(())
}

fn load_json(path: &PathBuf) -> Result<Value> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))?;
    let json = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {:?}", path))?;
    Ok(json)
}

fn compare_single_file(
    args: &Args,
    base_items: &[(String, Value)],
    compare_file: &PathBuf,
) -> Result<ComparisonResult> {
    let compare_json = load_json(compare_file)?;
    let mut missing_paths = Vec::new();
    let mut different_values = Vec::new();
    let mut type_mismatches = Vec::new();

    for (path, base_value) in base_items {
        match get_value_by_path(&compare_json, path) {
            None => {
                if !args.include_parents || !is_parent_missing(&missing_paths, path) {
                    missing_paths.push(path.clone());
                }
            }
            Some(compare_value) if args.check_values => {
                if args.type_only {
                    // only check types
                    if !same_type(base_value, compare_value) {
                        type_mismatches.push(TypeMismatch {
                            path: path.clone(),
                            base_type: get_value_type(base_value),
                            compare_type: get_value_type(compare_value),
                            base_value: base_value.clone(),
                            compare_value: compare_value.clone(),
                        });
                    }
                    // If the types are the same, we consider it a match even if values differ
                } else {
                    // check both type and value
                    if !values_equal(base_value, compare_value) {
                        different_values.push(ValueDifference {
                            path: path.clone(),
                            base_value: base_value.clone(),
                            compare_value: compare_value.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    let statistics = Statistics {
        total_paths_checked: base_items.len(),
        missing_count: missing_paths.len(),
        different_count: different_values.len(),
        type_mismatch_count: type_mismatches.len(),
        match_count: base_items.len() - missing_paths.len() - different_values.len(),
    };

    Ok(ComparisonResult {
        base_file: args.base_file.display().to_string(),
        compare_file: compare_file.display().to_string(),
        missing_paths,
        different_values,
        type_mismatches,
        statistics,
    })
}

fn print_header(args: &Args, base_items: &[(String, Value)]) {
    println!("{}", "═".repeat(80).bright_blue());
    println!("{}", "JSON Diff Checker".bright_white().bold());
    println!("{}", "═".repeat(80).bright_blue());
    println!(
        "Base file: {}",
        args.base_file.display().to_string().bright_yellow()
    );
    println!(
        "Total items: {}",
        base_items.len().to_string().bright_green()
    );
    println!(
        "Value checking: {}",
        if args.check_values {
            if args.type_only {
                "Type only".bright_cyan()
            } else {
                "Full comparison".bright_green()
            }
        } else {
            "Disabled".bright_black()
        }
    );
    println!("{}\n", "─".repeat(80).bright_black());
}

fn print_detailed_results(result: &ComparisonResult, args: &Args) {
    let path = PathBuf::from(&result.compare_file);
    let filename = path.file_name().unwrap_or_default().to_string_lossy();

    println!("{} {}", "▶".bright_blue(), filename.bright_white().bold());

    if !result.missing_paths.is_empty() {
        println!(
            "\n  {} Missing paths ({}):",
            "✗".red(),
            result.missing_paths.len()
        );
        for path in &result.missing_paths {
            println!("    {} {}", "└".bright_black(), path.bright_red());
        }
    }

    if args.check_values && !args.type_only && !result.different_values.is_empty() {
        println!(
            "\n  {} Different values ({}):",
            "≠".yellow(),
            result.different_values.len()
        );
        for diff in &result.different_values {
            println!("    {} {}", "└".bright_black(), diff.path.bright_yellow());
            println!(
                "      {} {}",
                "expected:".bright_black(),
                format_value(&diff.base_value).green()
            );
            println!(
                "      {} {}",
                "actual:  ".bright_black(),
                format_value(&diff.compare_value).red()
            );
        }
    }

    if args.check_values && args.type_only && !result.type_mismatches.is_empty() {
        println!(
            "\n  {} Type mismatches ({}):",
            "⚠".bright_magenta(),
            result.type_mismatches.len()
        );
        for mismatch in &result.type_mismatches {
            println!(
                "    {} {}",
                "└".bright_black(),
                mismatch.path.bright_magenta()
            );
            println!(
                "      {} {} → {}",
                "type:".bright_black(),
                mismatch.base_type.green(),
                mismatch.compare_type.red()
            );
        }
    }

    if result.missing_paths.is_empty()
        && result.different_values.is_empty()
        && result.type_mismatches.is_empty()
    {
        println!("  {} All items match!", "✓".bright_green());
    }

    println!();
}

fn print_summary(result: &ComparisonResult) {
    let path = PathBuf::from(&result.compare_file);
    let filename = path.file_name().unwrap_or_default().to_string_lossy();

    let status = if result.missing_paths.is_empty()
        && result.different_values.is_empty()
        && result.type_mismatches.is_empty()
    {
        format!("{} OK", "✓").bright_green().to_string()
    } else {
        let mut parts = vec![];
        if !result.missing_paths.is_empty() {
            parts.push(
                format!("{} missing", result.missing_paths.len())
                    .red()
                    .to_string(),
            );
        }
        if !result.different_values.is_empty() {
            parts.push(
                format!("{} different", result.different_values.len())
                    .yellow()
                    .to_string(),
            );
        }
        if !result.type_mismatches.is_empty() {
            parts.push(
                format!("{} type mismatch", result.type_mismatches.len())
                    .bright_magenta()
                    .to_string(),
            );
        }
        parts.join(", ")
    };

    println!("{:<30} {}", filename, status);
}

fn print_overall_summary(results: &[ComparisonResult]) {
    println!("{}", "─".repeat(80).bright_black());
    println!("{}", "Summary".bright_white().bold());
    println!("{}", "─".repeat(80).bright_black());

    let total_files = results.len();
    let perfect_matches = results
        .iter()
        .filter(|r| {
            r.missing_paths.is_empty()
                && r.different_values.is_empty()
                && r.type_mismatches.is_empty()
        })
        .count();
    let with_missing = results
        .iter()
        .filter(|r| !r.missing_paths.is_empty())
        .count();
    let with_different = results
        .iter()
        .filter(|r| !r.different_values.is_empty())
        .count();
    let with_type_mismatch = results
        .iter()
        .filter(|r| !r.type_mismatches.is_empty())
        .count();

    println!(
        "Total files checked: {}",
        total_files.to_string().bright_white()
    );
    println!(
        "Perfect matches: {}",
        perfect_matches.to_string().bright_green()
    );
    if with_missing > 0 {
        println!(
            "Files with missing paths: {}",
            with_missing.to_string().bright_red()
        );
    }
    if with_different > 0 {
        println!(
            "Files with different values: {}",
            with_different.to_string().bright_yellow()
        );
    }
    if with_type_mismatch > 0 {
        println!(
            "Files with type mismatches: {}",
            with_type_mismatch.to_string().bright_magenta()
        );
    }
}

fn format_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", s),
        Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}

fn export_results(path: &PathBuf, results: &[ComparisonResult]) -> Result<()> {
    let json = serde_json::to_string_pretty(results)?;
    fs::write(path, json)?;
    Ok(())
}
