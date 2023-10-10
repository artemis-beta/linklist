use std::process;
use colored::Colorize;
use regex::Regex;
use url::Url;

/// Retrieves the domain part of the user provided URL
pub fn get_domain(url: &String) -> String {
    let parsed_url = match Url::parse(url) {
        Ok(p) => p,
        Err(_) => panic!("Unable to parse URL")
    };
    match parsed_url.domain() {
        Some(p) => format!("{}://{}",  parsed_url.scheme(), p).to_string(),
        None => {
            return match parsed_url.host_str() {
                Some(v) => {
                    let port = match parsed_url.port() {
                        Some(k) => format!(":{}",  k).to_string(),
                        None => "".to_string()
                    };
                    format!("{}://{}{}", parsed_url.scheme(), v.to_string(), port)
                }
                None => panic!("Failed to retrieve domain")
            }
        }
    }
}

pub fn get_base_url(url: &String) -> String {
    let parsed_url = match Url::parse(url) {
        Ok(p) => p,
        Err(_) => panic!("Unable to parse URL")
    };
    let path_segments: Vec<&str> = parsed_url.path().split("/").collect::<Vec<&str>>();
    match path_segments.last(){
        Some(p) => {
            if p.contains(".") {
                return parsed_url.as_str().replace(p, "");
            }
            return url.clone();
        },
        None => url.clone()
    }
}


/// Parse the Web page HTML for file links
pub fn get_file_links(page_content: &String, domain: &String) -> Vec<String> {
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


/// Parse the Web page HTML for internal page links
pub fn get_page_links(page_content: &String, domain: &String) -> Vec<String> {
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


#[cfg(test)]
mod test {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    fn load_html_file_as_str() -> String {
        let mut resource_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        resource_dir.push("resources/demo_site.html");

        match fs::read_to_string(resource_dir) {
            Ok(s) => s,
            Err(_) => panic!("Failed to load file data")
        }
    }

    #[test]
    fn test_parse_page_links() -> () {
        let dummy_domain = "http://127.0.0.1:5500".to_string();
        assert_eq!(get_page_links(&load_html_file_as_str(), &dummy_domain), vec!["some_other_page", "/another_page", "/yet_another_page/"]);
    }

    #[test]
    fn test_parse_file_links() -> () {
        let dummy_domain = "http://127.0.0.1:5500".to_string();
        assert_eq!(get_file_links(&load_html_file_as_str(), &dummy_domain), vec!["/a_linked_file.html", "file_link.html"]);
    }

    #[test]
    fn test_get_base_url_normal() -> () {
        let dummy_url = "http://www.not_a_real_place.com/subdirectory/a_file.html".to_string();
        assert_eq!(get_base_url(&dummy_url), "http://www.not_a_real_place.com/subdirectory/");
    }

    #[test]
    fn test_get_base_url_ip_port() -> () {
        let dummy_url = "http://127.0.0.1:5050/subdirectory/a_file.html".to_string();
        assert_eq!(get_base_url(&dummy_url), "http://127.0.0.1:5050/subdirectory/");
    }

    #[test]
    fn test_get_domain_ip_port() -> () {
        let dummy_url = "http://127.0.0.1:5050/subdirectory/a_file.html".to_string();
        assert_eq!(get_domain(&dummy_url), "http://127.0.0.1:5050");
    }

    #[test]
    fn test_get_domain_normal() -> () {
        let dummy_url = "http://www.not_a_real_place.com/subdirectory/a_file.html".to_string();
        assert_eq!(get_domain(&dummy_url), "http://www.not_a_real_place.com");
    }
}