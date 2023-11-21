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

use clap::Parser;
use colored::Colorize;
use itertools::Itertools;
use reqwest;
use rpassword;
use secrecy::{ExposeSecret, Secret};
use std::{io::Write, process};

mod parsing;

#[derive(Parser, Debug)]
#[command(name = "linklist")]
#[command(author, version)]
#[command(about = "Website URL link lister")]
#[command(long_about = "Lists local links within a webpage by parsing for 'href' tags")]
struct CommandLineInterface {
    /// URL of Web page to parse
    url: String,

    /// Display as paths (without domain)
    #[arg(short, long, help = "Show as relative paths instead of full URL")]
    path: bool,

    /// Provide login credentials interactively
    #[arg(short, long, help = "Provide login credentials interatively")]
    login: bool,

    /// Specify username as argument
    #[arg(long, help = "Server login username")]
    user_name: Option<String>,

    /// Specify password as argument
    #[arg(long, help = "Server login password")]
    password: Option<String>,

    /// Allow invalid certificates
    #[arg(short, long, help = "Allow invalid certificates")]
    disable_ssl_verify: bool,

    /// Show file links in addition to web pages.
    /// If 'all' show all files, else filter by type, e.g. 'png', 'html' etc.
    #[arg(
        short,
        long,
        help = "Also list files of a particular type <all/png/..>"
    )]
    file_type: Option<String>,

    /// Colorise outputs by category
    #[arg(long, help = "Colorise results by category")]
    color: bool,
}

fn process_arguments(cli: CommandLineInterface) -> Result<(), String> {
    let domain_arg: String = if cli.url.starts_with("http") {
        cli.url.clone()
    } else {
        "http://".to_string() + &cli.url
    };

    let mut user_name = match cli.user_name {
        Some(u) => Secret::new(u),
        None => Secret::new("".to_string()),
    };
    let mut password = Secret::new(cli.password);

    if cli.login {
        print!("Username:{}", " ");
        std::io::stdout().flush().unwrap();
        if user_name.expose_secret().is_empty() {
            let mut read_user_str = "".to_string();
            user_name = Secret::new(
                std::io::stdin()
                    .read_line(&mut read_user_str)
                    .expect("Failed to read username")
                    .to_string(),
            );
        }
        password = match rpassword::prompt_password("Password: ") {
            Ok(o) => Secret::new(Some(o)),
            Err(_) => return Err(format!("Request for password failed")),
        };
    }

    let response: reqwest::blocking::Response = match reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(cli.disable_ssl_verify)
        .build()
        .unwrap()
        .get(&domain_arg)
        .header("Accept", "application/json")
        .header("User-Agent", "Rust")
        .basic_auth(user_name.expose_secret(), password.expose_secret().as_ref())
        .send()
    {
        Ok(val) => val,
        Err(e) => {
            return Err(format!(
                "{}{}{}{}",
                "Failed to retrieve site content for '",
                &domain_arg,
                "': ",
                e.to_string()
            ));
        }
    };

    let status: u16 = response.status().as_u16();

    let response_text: String = match response.text() {
        Ok(val) => val,
        Err(_) => {
            return Err(format!("{}", "Could not read site contents as text"));
        }
    };

    if status != reqwest::StatusCode::OK {
        return Err(format!("[{}] {}", status, response_text));
    }

    let mut links: Vec<String> = Vec::<String>::new();

    let domain_search = parsing::get_domain(&domain_arg);
    let mut base_url = parsing::get_base_url(&domain_arg);

    if base_url.ends_with("/") {
        match base_url.pop() {
            Some(_) => (),
            None => (),
        };
    }

    if cli.file_type.is_none() || cli.file_type.as_ref().unwrap() == "all" {
        let page_links: Vec<String> = parsing::get_page_links(&response_text, &domain_search);
        links.extend(page_links);
    }

    if cli.file_type.is_some() {
        let file_links: Vec<String> =
            parsing::get_file_links(&response_text, &domain_search, &cli.file_type.unwrap());
        links.extend(file_links);
    }

    let uniq_links: std::vec::IntoIter<&String> = links.iter().unique().sorted();

    if links.len() < 1 {
        return Err(format!("{}", "No links were found."));
    }

    for link in uniq_links {
        let print_link = format!(
            "{}{}{}",
            if cli.path { "" } else { &base_url },
            if link.starts_with("/") { "" } else { "/" },
            link
        );
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

    Ok(())
}

fn main() {
    let cli: CommandLineInterface = CommandLineInterface::parse();

    match process_arguments(cli) {
        Ok(_) => process::exit(0),
        Err(e) => {
            println!("{}", e.bold().red());
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use httpmock::MockServer;
    use rstest::*;
    use serde_json::json;
    use base64::{Engine, engine::general_purpose};

    use crate::process_arguments;

    #[rstest]
    #[case("valid".to_string(), "password".to_string(), true)]
    #[case("valid".to_string(), "badpass".to_string(), false)]
    fn test_linklist_with_login(#[case] user_name: String, #[case] password: String,#[case] pass: bool) -> Result<(), String> {
        let valid_username = "valid";
        let valid_password = "password";

        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method("GET")
                .path("/")
                .header("user-agent", "Rust")
                .header("Accept", "application/json")
                .header("authorization", format!("Basic {}", general_purpose::STANDARD.encode(format!("{}:{}", valid_username, valid_password))));
            then.status(201)
                .json_body(json!({"message": "Login successful", "token": "sometoken"}));
        });

        match process_arguments(CommandLineInterface {
            url: server.base_url().to_string(),
            path: false,
            login: false,
            user_name: Some(user_name),
            password: Some(password),
            disable_ssl_verify: false,
            file_type: None,
            color: false,
        }) {
            Ok(_) => {
                if pass {mock.assert();}
                if !pass {panic!("Test should have failed")} else {Ok(())}
            },
            Err(e) => {
                if pass {mock.assert()};
                if pass {Ok(())} else {panic!("Test should have passed but got '{}' for {}", e, server.base_url().as_str())}
            }
        }

    }
}
