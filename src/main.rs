//! # Website page link lister
//! 
//! <b>Lists all local page links from within a webpage by looking for all href tags within the HTML source.</b>
//! 
//! Linklister provides a quick and easy to use tool for getting all links which point to other pages on
//! the given domain. The output can be customised to include links to files.
//! 
//! ## Usage
//! ```sh
//! linklist <URL> [--color] [--path] [--file-type=<'all'/'png'/'html'/etc>]
//! ```
#![warn(missing_docs)]

use reqwest;
use clap::Parser;
use regex::Regex;
use colored::Colorize;
use std::process;
use itertools::Itertools;

#[derive(Parser,Debug)]
#[command(name="linklist")]
#[command(author,version)]
#[command(about="Website URL link lister")]
#[command(long_about = "Lists local links within a webpage by parsing for 'href' tags")]
struct CommandLineInterface {
    /// URL of Web page to parse
    url: String,

    /// Display as paths (without domain)
    #[arg(short, long, help="Show as relative paths instead of full URL")]
    path: bool,

    /// Show file links in addition to web pages. 
    /// If 'all' show all files, else filter by type, e.g. 'png', 'html' etc.
    #[arg(short, long, help="Also list files of a particular type <all/png/..>")]
    file_type: Option<String>,

    /// Colorise outputs by category
    #[arg(long, help="Colorise results by category")]
    color: bool
}


fn main() {
    let cli = CommandLineInterface::parse();
    let domain_arg = if cli.url.starts_with("http") {cli.url.clone()} else {"http://".to_string() + &cli.url};

    let response = match reqwest::blocking::Client::new()
        .get(&domain_arg)
        .header("Accept", "application/json")
        .header("User-Agent", "Rust")
        .send() {
            Ok(val) => val,
            Err(_) => {
                println!("{}{}{}", "Failed to retrieve site content for '".red().bold(), &domain_arg.red().bold(), "'".red().bold());
                process::exit(1);
            }
        };

    let status: u16 = response.status().as_u16();

    let response_text = match response.text() {
        Ok(val) => val,
        Err(_) => {
            println!("{}", "Could not read site contents as text".red().bold());
            process::exit(1);
        }
    };

    if status != reqwest::StatusCode::OK {
        println!("[{}] {}", status, response_text.red().bold());
        process::exit(1);
    }

    let re_domain = Regex::new(r"(https*://[\w\-\.\d]+)/*").unwrap();
    let re_href_internal = Regex::new(r#"href=["'](/*[\w\d_/\-]+/*)#*[\w\d_\-/]*\?*[\w\d_\-=&/]*["']"#).unwrap();
    let re_href_abs = Regex::new((r#"href=["']"#.to_string() + &domain_arg + r#"(/*[\w\d_/-]+/*)#*[\w\d_\-/]*\?*[\w\d_\-=&/]*["']"#).as_str()).unwrap();

    let re_href_file = Regex::new(r#"href=["'](/*[\w\d_/\-]+\.\w+*)#*[\w\d_\-/]*\?*[\w\d_\-=&/]*["']"#).unwrap();
    let re_href_file_abs = Regex::new((r#"href=["']"#.to_string() + &domain_arg + r#"(/*[\w\d_\-/]+\.\w+/*)#*[\w\d_\-/]*\?*[\w\d_\-=&/]*["']"#).as_str()).unwrap();

    let mut links: Vec::<String> = Vec::<String>::new();

    let domain_search = re_domain
    .captures_iter(&domain_arg)
    .filter_map(|cap| {
        let path = match cap.get(1) {
            Some(p) => p,
            None => {
                println!("{}", "Failed to retrieve domain from provided URL".red().bold());
                process::exit(1);
            }
        };
        Some(path.as_str())
    })
    .map(|m| m.to_string())
    .collect::<Vec<_>>();

    if domain_search.len() < 1 {
        println!("{}", "Failed to retrieve domain from provided URL".red().bold());
        process::exit(1);
    }

    if cli.file_type.is_none() || cli.file_type.as_ref().unwrap() == "all" {
        let mut href_internal_paths = re_href_internal
            .captures_iter(&response_text)
            .filter_map(|cap| {
                let path = match cap.get(1) {
                    Some(p) => p,
                    _ => panic!("Failed to retrieve regex capture group")
                };
                Some(path.as_str())
            })
            .map(|m| m.to_string())
            .collect::<Vec<_>>();
        
        let mut href_internal_abs = re_href_abs
            .captures_iter(&response_text)
            .filter_map(|cap| {
                let path = match cap.get(1) {
                    Some(p) => p,
                    _ => panic!("Failed to retrieve regex capture group")
                };
                Some(path.as_str())
            })
            .map(|m| m.to_string())
            .collect::<Vec<_>>();
            links.append(&mut href_internal_abs);
            links.append(&mut href_internal_paths);
    }

    if cli.file_type.is_some() {
        let mut href_internal_file_paths = re_href_file
            .captures_iter(&response_text)
            .filter_map(|cap| {
                let path = match cap.get(1) {
                    Some(p) => p,
                    _ => panic!("Failed to retrieve regex capture group")
                };
                Some(path.as_str())
            })
            .map(|m| m.to_string())
            .collect::<Vec<_>>();
        
        let mut href_internal_file_abs = re_href_file_abs
            .captures_iter(&response_text)
            .filter_map(|cap| {
                let path = match cap.get(1) {
                    Some(p) => p,
                    _ => panic!("Failed to retrieve regex capture group")
                };
                Some(path.as_str())
            })
            .map(|m| m.to_string())
            .collect::<Vec<_>>();
            links.append(&mut href_internal_file_paths);
            links.append(&mut href_internal_file_abs);
    }

    let uniq_links = links
        .iter()
        .unique()
        .sorted();

    if links.len() < 1 {
        println!("{}", "No links were found.".red().bold());
        process::exit(1);
    }

    for link in uniq_links {
        let domain = if !domain_search[0].ends_with("/") && !link.starts_with("/") {domain_search[0].to_string() + "/"} else {domain_search[0].to_string()};
        let print_link = format!("{}{}", if cli.path {""} else {&domain}, link);
        if cli.color {
            if link.ends_with("/") {
                println!("{}", print_link.blue().bold());
            } else if link.split(".").collect::<Vec<_>>().len() > 1 {
                println!("{}", print_link.cyan().bold());
            } else {
                println!("{}", print_link.bold());
            }
        } else {
            println!("{}", print_link);
        }
    }

    process::exit(0)

}
