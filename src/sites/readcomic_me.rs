#![allow(dead_code)]

use reqwest::blocking::Client;
use scraper::{selectable::Selectable, Html, Selector};
use std::path::{Path, PathBuf};

use super::*;

#[derive(Debug)]
pub struct ReadcomicMe {
    base_url: String,
    comic_url: String,
    client: Client,
    download_path: PathBuf,
}

impl SiteDownloader for ReadcomicMe {}

impl ReadcomicMe {
    pub fn new(comic_path: &str) -> Self {
        let base_url = "https://readcomic.me".to_string();
        let client = Client::new();
        let comic_url = comic_path.replace(&base_url, "");
        let download_path = Path::new(&comic_url.replace("/comic/", "")).to_owned();
        Self { base_url, comic_url, client, download_path }
    }
}

#[allow(unused_variables)]
impl SiteDownloaderFunctions for ReadcomicMe{
    fn download_issue(&self, issue_name: &Issue) -> Result<(), SiteDownloaderError> {
        return Ok(());
    }

    fn download_page(&self, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError> {
        return Ok(());
    }

    fn get_issues_list(&self, link: &str) ->Result<Vec<Issue>, SiteDownloaderError> {
        let mut vec = Vec::new();
        let response = self.client.get(link).send().unwrap();
        let body = response.text().unwrap();
        let document = Html::parse_document(&body);
        let selector = Selector::parse("#nt_listchapter").unwrap();
        let list = document.select(&selector).next();
        if list.is_none(){
            return Err(SiteDownloaderError::NotFound);
        }
        let link_selector = Selector::parse("a").unwrap();
        let links: Vec<_> = list.unwrap().select(&link_selector).collect();
        for link in links {
            let link_number: String = link.text()
                .collect::<String>()
                .lines().nth(1).expect("couldn't find the issue number")
                .replace("Issue #","");
            let issue_url: String = link.value().attr("href").unwrap().to_owned();
            let issue: Issue = Issue { name: link_number, link: issue_url };
            vec.push(issue);
        }
        return Ok(vec);
    }
}

