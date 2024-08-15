const OUT_DIR: &'static str = "docs/";
const IN_DIR: &'static str = "raw/";
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

fn main() {
    println!("clearing out {}", OUT_DIR);
    let _ = fs::remove_dir_all(OUT_DIR);
    let _ = fs::create_dir(OUT_DIR);
    let _ = fs::write(format!("{}/.nojekyll", OUT_DIR), "");
    let mut backlinks: HashMap<String, Vec<String>> = HashMap::new();
    let mut files: HashMap<String, String> = HashMap::new();

    let template = fs::read_to_string("template.html").expect("should exist");

    // find backlinks
    for entry in WalkDir::new(IN_DIR).into_iter().filter_map(|e| e.ok()) {
        let real_path = format!("{}", entry.path().display());
        let real_path = real_path
            .strip_prefix(IN_DIR)
            .expect("if it doesn't have it something has GONE WRONG");
        println!("found {}", real_path);
        if entry.path().is_dir() {
            let _ = fs::create_dir(format!("{}{}", OUT_DIR, real_path));
        } else if real_path.ends_with(".pssg") {
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
                        l.strip_suffix(" <--").expect("ends with")
                    );
                }
            }
            files.insert(real_path.to_string().replace("\\", "/"), f);
        }
    }
    for (_, l) in backlinks.iter_mut() {
        l.sort()
    }
    // insert backlinks
    let files: HashMap<String, String> = files
        .iter()
        .map(|(k, f)| {
            let mut new_content = String::new();
            if k != "index.pssg" {
                new_content.push_str("=> index.pssg\n\n");
            }
            for l in f.lines() {
                new_content.push_str(l);
                new_content.push('\n');
                if l.starts_with("--> ") {
                    let backlink_type = l.strip_prefix("--> ").expect("starts with it");
                    let my_backlinks = backlinks.get(backlink_type);
                    if my_backlinks.is_some() {
                        for l in my_backlinks.unwrap() {
                            println!("{} now links to {}", k, l);
                            new_content.push_str("=> ");
                            new_content.push_str(l);
                            new_content.push('\n');
                        }
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
                    let url = if url.contains("|") {
                        url.split("|").last().expect("it should exist")
                    } else {
                        url
                    };
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
                        let get_back_to_there = k
                            .strip_prefix(&common)
                            .expect("we know it starts with it")
                            .split("/")
                            .skip(1)
                            .map(|a| "../")
                            .collect::<Box<[&str]>>()
                            .join("");
                        let then_go = url
                            .strip_prefix(&common)
                            .expect("we know it starts with it");

                        let then_go = if then_go.ends_with(".pssg") {
                            format!("{}.html", then_go.strip_suffix(".pssg").expect("ends with"))
                        } else {
                            then_go.into()
                        };

                        let relative_url = format!("{}{}", get_back_to_there, then_go);

                        new_content.push_str("<a href=\"");
                        new_content.push_str(&relative_url);
                        new_content.push_str("\">");
                        new_content.push_str(l);
                        new_content.push_str("</a>\n");
                    }
                } else if l.starts_with("#") {
                    new_content.push_str("<b>");
                    new_content.push_str(l);
                    new_content.push_str("</b>\n")
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
        let out_thing = format!(
            "{}{}.html",
            OUT_DIR,
            k.strip_suffix(".pssg").expect("um i dunno")
        );
        let _ = fs::write(out_thing, template.replace("||content||", &v));
    }
}
