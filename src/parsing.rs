use std::process;
use colored::Colorize;
use regex::Regex;

/// Retrieves the domain part of the user provided URL
pub fn get_full_domain(url: &String) -> String {
    let re_domain = match Regex::new(r"(https*://[\w\-\.\d]+)/*")  {
        Ok(val) => val,
        Err(_) => {
            println!("{}", "Internal Error: Failed to parse regex string".red());
            process::exit(1);
        }
    };

    let domain_search: Vec<String> = re_domain
    .captures_iter(&url)
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

    domain_search[0]
}


/// Parse the Web page HTML for internal page links
pub fn get_page_links(page_content: &String, domain: &String) -> Vec<String> {
    let mut links: Vec<String> = Vec::<String>::new();

    let re_href_file = match Regex::new(r#"href=["'](/*[\w\d_/\-]+\.\w+*)#*[\w\d_\-/]*\?*[\w\d_\-=&/]*["']"#) {
        Ok(val) => val,
        Err(_) => {
            println!("{}", "Internal Error: Failed to parse regex string".red());
            process::exit(1);
        }
    };
    let re_href_file_abs = match Regex::new((r#"href=["']"#.to_string() + domain + r#"(/*[\w\d_\-/]+\.\w+/*)#*[\w\d_\-/]*\?*[\w\d_\-=&/]*["']"#).as_str()) {
        Ok(val) => val,
        Err(_) => {
            println!("{}", "Internal Error: Failed to parse regex string".red());
            process::exit(1);
        }
    };

    let mut href_internal_paths = re_href_file
        .captures_iter(&page_content)
        .filter_map(|cap| {
            let path = match cap.get(1) {
                Some(p) => p,
                _ => panic!("Failed to retrieve regex capture group")
            };
            Some(path.as_str())
        })
        .map(|m| m.to_string())
        .collect::<Vec<_>>();
    
    let mut href_internal_abs = re_href_file_abs
        .captures_iter(&page_content)
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
    
    links
}


/// Parse the Web page HTML for internal file links
pub fn get_file_links(page_content: &String, domain: &String) -> Vec<String> {
    let mut links: Vec<String> = Vec::<String>::new();

    let re_href_internal = match Regex::new(r#"href=["'](/*[\w\d_/\-]+/*)#*[\w\d_\-/]*\?*[\w\d_\-=&/]*["']"#) {
        Ok(val) => val,
        Err(_) => {
            println!("{}", "Internal Error: Failed to parse regex string".red());
            process::exit(1);
        }
    };
    let re_href_abs = match Regex::new((r#"href=["']"#.to_string() + domain + r#"(/*[\w\d_/-]+/*)#*[\w\d_\-/]*\?*[\w\d_\-=&/]*["']"#).as_str()) {
        Ok(val) => val,
        Err(_) => {
            println!("{}", "Internal Error: Failed to parse regex string".red());
            process::exit(1);
        }
    };

    let mut href_internal_file_paths = re_href_internal
        .captures_iter(&page_content)
        .filter_map(|cap| {
            let path = match cap.get(1) {
                Some(p) => p,
                _ => panic!("Failed to retrieve regex capture group")
            };
            Some(path.as_str())
        })
        .map(|m| m.to_string())
        .collect::<Vec<_>>();
    
    let mut href_internal_file_abs = re_href_abs
        .captures_iter(&page_content)
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
    
    links
}