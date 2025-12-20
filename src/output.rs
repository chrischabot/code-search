use anyhow::Result;
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::cli::{OutputFormat, SearchArgs};

#[derive(Clone, Serialize)]
pub struct SearchResult {
    pub path: PathBuf,
    pub line_number: usize,
    pub column: usize,
    pub line_content: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub context_before: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub context_after: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Serialize)]
struct JsonOutput {
    total_matches: usize,
    results: Vec<SearchResult>,
}

pub fn display_results(results: &[SearchResult], args: &SearchArgs) -> Result<()> {
    match args.output {
        OutputFormat::Human => display_human(results, args),
        OutputFormat::Json => display_json(results),
    }
}

fn display_human(results: &[SearchResult], args: &SearchArgs) -> Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);

    if args.count {
        // Count mode - show file:count
        for result in results {
            let mut path_color = ColorSpec::new();
            path_color.set_fg(Some(Color::Magenta));
            
            stdout.set_color(&path_color)?;
            write!(stdout, "{}", result.path.display())?;
            stdout.reset()?;
            writeln!(stdout, ":{}", result.line_number)?;
        }
        return Ok(());
    }

    if args.files_only {
        // Files only mode
        for result in results {
            let mut path_color = ColorSpec::new();
            path_color.set_fg(Some(Color::Magenta));
            
            stdout.set_color(&path_color)?;
            writeln!(stdout, "{}", result.path.display())?;
            stdout.reset()?;
        }
        return Ok(());
    }

    let mut prev_path: Option<&PathBuf> = None;
    
    for result in results {
        // Print file header if path changed
        if prev_path != Some(&result.path) {
            if prev_path.is_some() {
                writeln!(stdout)?; // Blank line between files
            }
            
            let mut path_color = ColorSpec::new();
            path_color.set_fg(Some(Color::Magenta)).set_bold(true);
            
            stdout.set_color(&path_color)?;
            writeln!(stdout, "{}", result.path.display())?;
            stdout.reset()?;
            
            prev_path = Some(&result.path);
        }

        // Print context before
        for (i, line) in result.context_before.iter().enumerate() {
            let ctx_line_num = result.line_number - result.context_before.len() + i;
            print_context_line(&mut stdout, ctx_line_num, line)?;
        }

        // Print matching line
        print_match_line(&mut stdout, result.line_number, &result.line_content, result.score)?;

        // Print context after
        for (i, line) in result.context_after.iter().enumerate() {
            let ctx_line_num = result.line_number + 1 + i;
            print_context_line(&mut stdout, ctx_line_num, line)?;
        }

        // Separator if there's context
        if !result.context_before.is_empty() || !result.context_after.is_empty() {
            let mut sep_color = ColorSpec::new();
            sep_color.set_fg(Some(Color::Cyan));
            stdout.set_color(&sep_color)?;
            writeln!(stdout, "--")?;
            stdout.reset()?;
        }
    }

    // Print summary
    writeln!(stdout)?;
    let mut summary_color = ColorSpec::new();
    summary_color.set_fg(Some(Color::Green));
    stdout.set_color(&summary_color)?;
    writeln!(stdout, "Found {} matches", results.len())?;
    stdout.reset()?;

    Ok(())
}

fn print_match_line(stdout: &mut StandardStream, line_num: usize, content: &str, score: Option<f64>) -> Result<()> {
    let mut line_num_color = ColorSpec::new();
    line_num_color.set_fg(Some(Color::Green)).set_bold(true);
    
    stdout.set_color(&line_num_color)?;
    write!(stdout, "{:>6}", line_num)?;
    stdout.reset()?;
    
    let mut sep_color = ColorSpec::new();
    sep_color.set_fg(Some(Color::Cyan));
    stdout.set_color(&sep_color)?;
    write!(stdout, ":")?;
    stdout.reset()?;

    writeln!(stdout, " {}", content)?;

    // Print score if available (semantic search)
    if let Some(s) = score {
        let mut score_color = ColorSpec::new();
        score_color.set_fg(Some(Color::Yellow)).set_dimmed(true);
        stdout.set_color(&score_color)?;
        writeln!(stdout, "       [score: {:.3}]", s)?;
        stdout.reset()?;
    }

    Ok(())
}

fn print_context_line(stdout: &mut StandardStream, line_num: usize, content: &str) -> Result<()> {
    let mut line_num_color = ColorSpec::new();
    line_num_color.set_fg(Some(Color::Blue)).set_dimmed(true);
    
    stdout.set_color(&line_num_color)?;
    write!(stdout, "{:>6}", line_num)?;
    stdout.reset()?;
    
    let mut sep_color = ColorSpec::new();
    sep_color.set_fg(Some(Color::Cyan)).set_dimmed(true);
    stdout.set_color(&sep_color)?;
    write!(stdout, "-")?;
    stdout.reset()?;

    let mut content_color = ColorSpec::new();
    content_color.set_dimmed(true);
    stdout.set_color(&content_color)?;
    writeln!(stdout, " {}", content)?;
    stdout.reset()?;

    Ok(())
}

fn display_json(results: &[SearchResult]) -> Result<()> {
    let output = JsonOutput {
        total_matches: results.len(),
        results: results.to_vec(),
    };

    let json = serde_json::to_string_pretty(&output)?;
    println!("{}", json);
    
    Ok(())
}