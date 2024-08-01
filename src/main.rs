use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::collections::VecDeque;
use std::{env, thread};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use regex::Regex;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

mod sites;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parsing input
    let args: Vec<String> = env::args().collect();
    let url: String;
    if args.len() < 2 {
        println!("insert link to comic: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("couldn't read from terminal");
        url = input.trim().to_string();
        let comicdwl = sites::identify_website(&url).unwrap();
        println!("{:#?}", comicdwl);
        let issue_list = comicdwl.get_issues_list(&url)?;
        for issue in issue_list {
            println!("{:#?}", issue);
            comicdwl.download_issue(&issue)?;
        }
        return Ok(());
    } else {
        url = args[1].clone();
    }

    let comic_name = url.replace("https://readcomic.me/comic/", "");

    let client = Client::new();
    let response = client.get(url).send()?;
    if !response.status().is_success() {
        eprintln!("Error downloading the content: {:?}", response.status());
        std::process::exit(1);
    }

    let body = response.text()?;
    let document = Html::parse_document(&body);

    let selector = Selector::parse("#nt_listchapter").unwrap();
    let table = document.select(&selector).next();

    if table.is_none() {
        eprintln!("Couldn't find the issues");
        std::process::exit(1);
    }

    let link_selector = Selector::parse("a").unwrap();
    let links: Vec<_> = table.unwrap().select(&link_selector).collect();

    println!("Downloading {}", comic_name);
    if !Path::new(&comic_name).exists() {
        fs::create_dir(&comic_name)?;
    }

    let args: Vec<_> =  std::env::args().collect();
    let number_of_jobs = args.iter().filter(|e| e.starts_with("-J")).last();
    match number_of_jobs{
        Some(jobs_argument) => {
            let jobs_quantity: usize = jobs_argument.replace("-J", "").parse()?;
            println!("starting download with {jobs_quantity} threads" );
            let mut handles: VecDeque<thread::JoinHandle<()>> = VecDeque::new();
            for link in links {
                if handles.len() == jobs_quantity {
                    if let Some(handle) = handles.pop_front() {
                        handle.join().unwrap();
                    }
                }
                let client_clone = client.clone();
                let comic_name_clone = comic_name.to_string();
                let link_clone = link.value().attr("href").unwrap().to_string().clone();
                let linkhtml = link.inner_html().clone();

                let handle = thread::spawn(move  || {
                    let _ = download_issue(&client_clone, &comic_name_clone, &link_clone, linkhtml);

                });

                handles.push_back(handle);
            }

            for handle in handles{
                handle.join().unwrap();
            }


        },
        None => {
            for link in links {
                match download_issue(&client, &comic_name, link.value().attr("href").unwrap(), link.inner_html()) {
                    Ok(_) => {},
                    Err(_) => {println!("couldn't download something")},
                }
            }
        },
    }
    Ok(())
}

fn download_issue(client: &Client, comic_name: &str, issue_link: &str, issue_link_text: String) -> Result<(), Box<dyn std::error::Error>> {
    let pattern = Regex::new(r"#(\d+)")?;
    let issue_number = pattern.captures(&issue_link_text).unwrap().get(1).unwrap().as_str();
    println!("Downloading {} issue number {}", comic_name, issue_number);
    let issue_path = format!("{}/{}", comic_name, issue_number);
    if Path::new(&issue_path).exists() {
        println!("Was already downloaded");
        return Ok(());
    }
    fs::create_dir(&issue_path)?;

    let page = client.get(issue_link).send()?;
    let page_body = page.text()?;
    let document = Html::parse_document(&page_body);
    let total_pages_selector = Selector::parse("span.total-pages").unwrap();
    let total_pages_text = document.select(&total_pages_selector).next().unwrap().inner_html();
    let second_pattern = Regex::new(r"(\d+)")?;
    let number_of_pages = second_pattern.captures(&total_pages_text).unwrap().get(1).unwrap().as_str().parse::<i32>()?;
    println!("There are {} pages", number_of_pages);

    for n in 0..number_of_pages {
        let page_link = format!("{}/{}", issue_link, n + 1);
        download_page(&client, &issue_path, &page_link, n + 1)?;
    }
    create_cbz(&issue_path, &format!("{}-{}.cbz", comic_name, issue_number), comic_name)?;

    Ok(())
}

fn download_page(client: &Client, path: &str, link: &str, page_number: i32) -> Result<(), Box<dyn std::error::Error>> {
    let page = client.get(link).send()?;
    let page_body = page.text()?;
    let document = Html::parse_document(&page_body);
    let img_selector = Selector::parse("img.single-page").unwrap();
    let page_image_link = document.select(&img_selector).next().unwrap().value().attr("src").unwrap();
    let page_name = format!("{:04}.jpg", page_number);
    let dw_path = format!("{}/{}", path, page_name);
    let mut response = client.get(page_image_link).send()?;
    if response.status().is_success() {
        let mut file = File::create(dw_path)?;
        io::copy(&mut response, &mut file)?;
    } else {
        eprintln!("Couldn't download page");
    }

    Ok(())
}

fn create_cbz(path: &str, output_filename: &str, comic_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let out_path = format!("{}/{}", comic_name, output_filename);
    let file = File::create(&out_path)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let files: Vec<_> = fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.path().to_str().map(|s| s.to_string()))
        .collect();

    for filename in files {
        let name = Path::new(&filename).file_name().unwrap().to_str().unwrap();
        let mut f = File::open(&filename)?;
        zip.start_file(name, options)?;
        io::copy(&mut f, &mut zip)?;
    }
    zip.finish()?;

    Ok(())
}
