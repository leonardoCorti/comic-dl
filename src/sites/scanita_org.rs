#![allow(dead_code)]

use std::{fs, path::PathBuf};

use regex::Regex;
use reqwest::blocking::Client;
use scraper::{selectable::Selectable, Html, Selector};

use super::*;

#[derive(Debug, Clone)]
pub struct ScanitaOrg {
    base_url: String,
    comic_url: String,
    client: Client,
    download_path: PathBuf,
    comic_name: String,
}

impl ScanitaOrg {
    pub fn new(comic_path: &str) -> Self {
        let base_url = "https://scanita.org".to_string();
        let client = Client::new();
        let comic_url = comic_path.replace(&base_url, "");
        let download_path: PathBuf = Path::new(&comic_url.replace("/manga/", "").as_str()).into();
        if !download_path.exists() {
            fs::create_dir(&download_path).unwrap();
        }
        let comic_name = comic_path.replace("https://scanita.org/manga/", "").into();
        Self { base_url, comic_url, client, download_path, comic_name }
    }
}

impl SiteDownloader for ScanitaOrg {}

#[allow(unused_variables)]
impl SiteDownloaderFunctions for ScanitaOrg {
    fn download_issue(&self, issue: &Issue) -> Result<(), SiteDownloaderError> {
        todo!()
    }

    fn download_page(&self, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError> {
        todo!()
    }

    fn get_issues_list(&self, link: &str) -> Result<Vec<Issue>, SiteDownloaderError> {
        let mut list_of_issues: Vec<Issue> = Vec::new();
        let body = self.client.get(link).send().unwrap()
            .text().unwrap();
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[data-show-more='#more-chapter']").unwrap();
        let list = document.select(&selector).next();
        match list {
            Some(button) => {
                let link_to_list = self.base_url.clone() + button.attr("data-path").unwrap();
                let chapters_body = self.client.get(link_to_list).send().unwrap().text().unwrap();
                let chapters_document = Html::parse_document(&chapters_body);

                let a_selector = Selector::parse("a[href]").unwrap();
                let h5_selector = Selector::parse("h5").unwrap();
                for a_chapter in chapters_document.select(&a_selector) {
                    let href= a_chapter.value().attr("href").unwrap();
                    let issue_link = self.base_url.clone() + href;
                    let chapter_text= a_chapter.select(&h5_selector)
                        .next().unwrap().text().collect::<Vec<_>>().concat();
                    let re= Regex::new(r"Capitolo (\d+)").unwrap();
                    let chapter_number: usize = re.captures(&chapter_text).unwrap().get(1).unwrap().as_str().parse().unwrap();
                    let chapter_name = format!("{:03}", chapter_number);

                    let issue: Issue = Issue { name: chapter_name, link: issue_link };
                    list_of_issues.push(issue);
                }
            },
            None => { //few chapters, no dedicated button
                println!("not found");
                todo!()
            },
        }
        list_of_issues.iter().for_each(|e| println!("{:#?}", e));
        return Ok(list_of_issues);
    }

    fn create_cbz(&self, issue_name: &Issue, issue_path: PathBuf) -> Result<(), SiteDownloaderError> {
        todo!()
    }
}

