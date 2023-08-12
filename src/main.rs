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

mod parsing;

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
    let cli: CommandLineInterface = CommandLineInterface::parse();
    let domain_arg: String = if cli.url.starts_with("http") {cli.url.clone()} else {"http://".to_string() + &cli.url};

    let response: reqwest::blocking::Response = match reqwest::blocking::Client::new()
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

    let response_text: String = match response.text() {
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

    let mut links: Vec::<String> = Vec::<String>::new();

    if cli.file_type.is_none() || cli.file_type.as_ref().unwrap() == "all" {
        let page_links: Vec<String> = parsing::get_page_links(&response_text, &domain_arg);
        links.extend(page_links);
    }

    if cli.file_type.is_some() {
        let file_links: Vec<String> = parsing::get_file_links(&response_text, &domain_arg);
        links.extend(file_links);
    }

    let uniq_links: std::vec::IntoIter<&String> = links
        .iter()
        .unique()
        .sorted();

    if links.len() < 1 {
        println!("{}", "No links were found.".red().bold());
        process::exit(1);
    }

    let domain_search = parsing::get_full_domain(&domain_arg);

    for link in uniq_links {
        let domain = if !domain_search.ends_with("/") && !link.starts_with("/") {domain_search.to_string() + "/"} else {domain_search[0].to_string()};
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
