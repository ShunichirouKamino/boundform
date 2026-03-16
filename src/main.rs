//! boundform — validate HTML form constraints against a YAML spec.

mod comparator;
mod config;
mod error;
mod model;
mod parser;
mod reporter;
mod source;

use clap::{Parser, ValueEnum};
use comparator::{PageResult, ValidationReport};
use reporter::OutputFormat;
use std::process;

#[derive(Parser)]
#[command(
    name = "boundform",
    about = "A fast guardrail before E2E — validate HTML form constraints against a YAML spec",
    version
)]
struct Cli {
    /// Path to the YAML config file
    #[arg(long, default_value = "boundform.yml")]
    config: String,

    /// Output format
    #[arg(long, default_value = "terminal")]
    format: Format,

    /// Cookie to attach to HTTP requests (can be specified multiple times)
    #[arg(long)]
    cookie: Vec<String>,

    /// Custom header to attach to HTTP requests, e.g. "Authorization: Bearer xxx" (can be specified multiple times)
    #[arg(long)]
    header: Vec<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum Format {
    Terminal,
    Json,
}

fn main() {
    let cli = Cli::parse();

    let options = source::FetchOptions {
        cookies: cli.cookie,
        headers: cli.header,
    };

    if let Err(e) = run_validate(&cli.config, &cli.format, &options) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run_validate(
    config_path: &str,
    format: &Format,
    options: &source::FetchOptions,
) -> error::Result<()> {
    let cfg = config::load_config(config_path)?;

    let output_format = match format {
        Format::Terminal => OutputFormat::Terminal,
        Format::Json => OutputFormat::Json,
    };

    let mut pages = Vec::new();

    for page_config in &cfg.pages {
        let html = source::fetch_html(&page_config.url, options)?;
        let actual_forms = parser::parse_forms(&html);
        let form_results = comparator::compare_forms(&page_config.forms, &actual_forms, &html);

        pages.push(PageResult {
            url: page_config.url.clone(),
            form_results,
        });
    }

    let report = ValidationReport { pages };
    let output = reporter::render_validation_report(&report, &output_format);
    print!("{output}");

    if comparator::report_has_errors(&report) {
        process::exit(1);
    }

    Ok(())
}
