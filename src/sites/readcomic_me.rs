use regex::Regex;
use reqwest::blocking::Client;
use scraper::{selectable::Selectable, Html, Selector};
use std::{fs::{self, File}, io, path::{Path, PathBuf}};

use super::*;

#[derive(Debug, Clone)]
pub struct ReadcomicMe {
    _base_url: String,
    comic_url: String,
    client: Client,
    download_path: PathBuf,
    comic_name: String,
}

impl SiteDownloader for ReadcomicMe {}

impl ReadcomicMe {
    pub fn new(comic_path: &str) -> Self {
        let base_url = "https://readcomic.me".to_string();
        let client = Client::new();
        let comic_url = comic_path.replace(&base_url, "");
        let download_path = Path::new(&comic_url.replace("/comic/", "")).to_owned();
        let comic_name = comic_path.replace("https://readcomic.me/comic/", "");
        Self { _base_url: base_url, comic_url, client, download_path, comic_name }
    }

    fn get_page_with_issues(&self, page_link: String) -> Option<String> {
        let response = self.client.get(page_link).send().unwrap();
        let body = response.text().unwrap();
        let document = Html::parse_document(&body);
        let selector = Selector::parse("#nt_listchapter").unwrap();
        let list = document.select(&selector).next();
        if list.is_none(){
            return None;
        }
        let link_selector = Selector::parse("a").unwrap();
        let links: Vec<_> = list.unwrap().select(&link_selector).collect();
        if links.iter().filter(|e| e.inner_html().contains("Issue #")).count() ==0 {
            return None;
        }
        return Some(body);
    }
}

impl SiteDownloaderFunctions for ReadcomicMe{
    fn download_issue(&self, issue_name: &Issue) -> Result<(), SiteDownloaderError> {
        if !self.download_path.exists() {
            if fs::create_dir(&self.download_path).is_err() {
                if !self.download_path.exists(){
                    return Err(SiteDownloaderError::FileSystemError);
                }
            };
        }
        let issue_link = &issue_name.link;
        let issue_number = &issue_name.name;
        println!("Downloading {} issue number {}", self.comic_url, issue_number);
        let issue_path = self.download_path.join(issue_number);
        let out_filename = format!("{}-{}.cbz", self.comic_name, issue_name.name);
        let out_path = self.download_path.join(&out_filename);
        if Path::new(&out_path).exists() {
            println!("Was already downloaded");
            return Ok(());
        }
        if !issue_path.exists() {
            fs::create_dir(&issue_path).unwrap();
        }

        let page = self.client.get(issue_link).send().unwrap();
        let page_body = page.text().unwrap();
        let document = Html::parse_document(&page_body);
        let total_pages_selector = Selector::parse("span.total-pages").unwrap();
        let total_pages_text = document.select(&total_pages_selector).next().unwrap().inner_html();
        let second_pattern = Regex::new(r"(\d+)").unwrap();
        let number_of_pages = second_pattern.captures(&total_pages_text).unwrap().get(1).unwrap().as_str().parse::<i32>().unwrap();
        println!("There are {} pages", number_of_pages);

        for n in 0..number_of_pages {
            let page_link = format!("{}/{}", issue_link, n + 1);
            self.download_page(&page_link, &issue_path, (n+1).try_into().unwrap()).unwrap();
        }

        self.create_cbz(issue_name, issue_path)?;

        return Ok(());
    }

    fn download_page(&self, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError> {
        let page = self.client.get(link).send().unwrap();
        let page_body = page.text().unwrap();
        let document = Html::parse_document(&page_body);
        let img_selector = Selector::parse("img.single-page").unwrap();
        let page_image_link = document.select(&img_selector).next().unwrap().value().attr("src").unwrap();
        let page_name = format!("{:04}.jpg", page_number);
        let dw_path =  issue_path.join(page_name);
        let mut response = self.client.get(page_image_link).send().unwrap();
        if response.status().is_success() {
            let mut file = File::create(dw_path).unwrap();
            io::copy(&mut response, &mut file).unwrap();
        } else {
            eprintln!("Couldn't download page");
        }
        return Ok(());
    }

    fn get_issues_list(&self, link: &str) ->Result<Vec<Issue>, SiteDownloaderError> {
        let mut vec = Vec::new();
        let mut page_number = 1;
        let mut page_link = link.to_string() + "?page="+&page_number.to_string();
        while let Some(page_with_link) = self.get_page_with_issues(page_link) {
            let document = Html::parse_document(&page_with_link);
            let selector = Selector::parse("#nt_listchapter").unwrap();
            let list = document.select(&selector).next();
            if list.is_none(){
                return Err(SiteDownloaderError::NotFound);
            }
            let link_selector = Selector::parse("a").unwrap();
            let links: Vec<_> = list.unwrap().select(&link_selector).collect();
            for link in links {
                let link_number: String = match link.inner_html()
                    .lines().nth(1){
                    Some(inner_line) => inner_line.replace("Issue #", "") ,
                    None => break,
                };
                let issue_url: String = link.value().attr("href").unwrap().to_owned();
                let issue: Issue = Issue { name: link_number, link: issue_url };
                vec.push(issue);
            }
            page_number += 1;
            page_link = link.to_string() + "?page="+&page_number.to_string();
        }
        vec.reverse();
        return Ok(vec);
    }

    fn create_cbz(&self, issue_name: &Issue, issue_path: PathBuf) -> Result<(), SiteDownloaderError> {
        let out_filename = format!("{}-{}.cbz", self.comic_name, issue_name.name);
        let out_path = self.download_path.join(out_filename);
        let file = File::create(&out_path).expect("error creating cbz");
        let mut zip = zip::ZipWriter::new(file);

        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let files: Vec<_> = fs::read_dir(&issue_path).expect("error creating cbz")
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.path().to_str().map(|s| s.to_string()))
            .collect();

        for filename in files {
            let name = Path::new(&filename).file_name().unwrap().to_str().unwrap();
            let mut f = File::open(&filename).expect("error creating cbz");
            zip.start_file(name, options).expect("error creating cbz");
            io::copy(&mut f, &mut zip).expect("error creating cbz");
        }
        zip.finish().expect("error creating cbz");
        fs::remove_dir_all(&issue_path).expect("couldn't clean source directory");
        return Ok(());
    }
    fn change_path(&mut self, new_path: &str) -> Result<(), SiteDownloaderError> {
        let new_path = Path::new(new_path);
        if !new_path.exists(){
            println!("The directory doesn't exists");
            return Err(SiteDownloaderError::FileSystemError);
        }
        let final_path = new_path.join(self.download_path.clone());
        self.download_path = final_path;
        return Ok(());
    }

    fn get_comic_name(&self) -> &str {
        return &self.comic_name;
    }
}

