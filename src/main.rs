use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::env;
use std::fs::{self, File};
use std::io::{self};
use std::path::Path;
use regex::Regex;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if an input argument is provided
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("No input provided. Halting the script.");
        std::process::exit(1);
    }

    // URL to download
    let url = &args[1];
    let comic_name = url.replace("https://readcomic.me/comic/", "");

    let client = Client::new();
    // Download the content
    let response = client.get(url).send()?;
    if !response.status().is_success() {
        eprintln!("Error downloading the content: {:?}", response.status());
        std::process::exit(1);
    }

    // Parse the HTML content
    let body = response.text()?;
    let document = Html::parse_document(&body);

    // Find the table with class "listings"
    let selector = Selector::parse("#nt_listchapter").unwrap();
    let table = document.select(&selector).next();

    if table.is_none() {
        eprintln!("Couldn't find the issues");
        std::process::exit(1);
    }

    // Find all link tags within the table
    let link_selector = Selector::parse("a").unwrap();
    let links: Vec<_> = table.unwrap().select(&link_selector).collect();

    // Output the links
    println!("Downloading {}", comic_name);
    if !Path::new(&comic_name).exists() {
        fs::create_dir(&comic_name)?;
    }

    for link in links {
        download_issue(&client, &comic_name, link.value().attr("href").unwrap(), link.inner_html())?;
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

    //let options= FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
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
