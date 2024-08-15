const OUT_DIR: &'static str = "docs/";
const IN_DIR: &'static str = "raw/";
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

fn main() {
    println!("clearing out {}", OUT_DIR);
    let _ = fs::remove_dir_all(OUT_DIR);
    let _ = fs::create_dir(OUT_DIR);
    let mut backlinks: HashMap<String, Vec<String>> = HashMap::new();
    let mut files: HashMap<String, String> = HashMap::new();

    // find backlinks
    for entry in WalkDir::new(IN_DIR).into_iter().filter_map(|e| e.ok()) {
        let real_path = format!("{}", entry.path().display());
        let real_path = real_path.strip_prefix(IN_DIR).expect("balls");
        println!("finding backlinks in {}", real_path);
        if entry.path().is_dir() {
            let _ = fs::create_dir(format!("{}{}", OUT_DIR, real_path));
        } else {
            let f = fs::read_to_string(entry.path())
                .unwrap_or("this is an absolutely FUCKED UP file".into())
                .replace("\r\n", "\n");
            // println!("{}", f);
            for l in f.lines() {
                if l.ends_with(" <--") {
                    let this_backlinks = backlinks
                        .entry(l.strip_suffix(" <--").expect("it ends with it").to_string())
                        .or_insert(Vec::new());
                    this_backlinks.push(real_path.to_string().replace("\\", "/"));
                    println!(
                        "backlink of type {}! taking note",
                        l.strip_suffix(" <--").expect("ski")
                    );
                }
            }
            files.insert(real_path.to_string().replace("\\", "/"), f);
        }
    }
    // insert backlinks
    let files: HashMap<String, String> = files
        .iter()
        .map(|(k, f)| {
            let mut new_content = String::new();
            for l in f.lines() {
                new_content.push_str(l);
                new_content.push('\n');
                if l.starts_with("--> ") {
                    let backlink_type = l.strip_prefix("--> ").expect("starts with it");
                    let my_backlinks = backlinks.get(backlink_type).expect("shitball");
                    for l in my_backlinks {
                        println!("{} now links to {}", k, l);
                        new_content.push_str("=> ");
                        new_content.push_str(l);
                        new_content.push('\n');
                    }
                }
            }
            (k.clone(), new_content)
        })
        .collect();

    // process regular links and convert to actual html
    let files: HashMap<String, String> = files
        .iter()
        .map(|(k, f)| {
            let mut new_content = String::new();
            for l in f.lines() {
                if l.starts_with("=> ") {
                    let url = l.strip_prefix("=> ").expect("starts with");
                    if url.starts_with("http") {
                        new_content.push_str("<a href=\"");
                        new_content.push_str(url);
                        new_content.push_str("\">");
                        new_content.push_str(l);
                        new_content.push_str("</a>\n");
                    } else {
                        // ah shit. we need to find the earliest folder which we share
                        let mut common = String::new();
                        for (l, r) in std::iter::zip(k.split("/"), url.split("/")) {
                            println!("{} v {}", l, r);
                            if l == r {
                                common.push_str(l);
                                common.push('/');
                            } else {
                                break;
                            }
                        }
                        println!("shared prefix: {}", common);
                    }
                } else {
                    new_content.push_str(l);
                    new_content.push('\n');
                }
            }
            (k.clone(), new_content)
        })
        .collect();

    for (k, v) in files {
        println!("===\n{}\n{}", k, v);
    }
}