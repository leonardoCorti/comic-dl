use std::{fs::{self, File}, path::PathBuf};

use reqwest::blocking::Client;
use scraper::{selectable::Selectable, Html, Selector};

use super::*;

#[derive(Debug, Clone)]
pub struct ScanitaOrgStrategy;

impl ComicDownloader for ScanitaOrgStrategy{
    fn download_issue(&self, client: &Client, download_path: &PathBuf, issue: &Issue) -> Result<(), SiteDownloaderError> {
        if !download_path.exists() {
            if fs::create_dir(&download_path).is_err() {
                if !download_path.exists(){
                    return Err(SiteDownloaderError::FileSystemError);
                }
            };
        }
        println!("downloading {}", issue.name);
        let issue_path = download_path.join(issue.name.clone());
        if !issue_path.exists(){
            fs::create_dir(&issue_path).unwrap();
        }
        let mut page_number: u32 = 1;
        loop {
            let link = issue.link.clone() + "/" + &page_number.to_string();
            let request = client.get(link).send().expect("couldn't donwload page");
            if request.status() == reqwest::StatusCode::FOUND {break;}
            let page =  request.text().unwrap();
            let document = Html::parse_document(&page);
            let page_selector = Selector::parse(".book-page").unwrap();
            let img_selector = Selector::parse("img").unwrap();
            let page_div = document.select(&page_selector).next();
            if page_div.is_none() {break;}
            let page_img = page_div.unwrap().select(&img_selector).next().unwrap();
            let page_img_link = page_img.attr("src").unwrap();
            self.download_page(client, page_img_link, &issue_path , page_number)?;
            page_number += 1;
        }
        return Ok(());
    }

    fn download_page(&self, client: &Client, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError> {
        let response = client.get(link).send().expect("can't download a page");
        let file_path = issue_path.join(format!("{:04}.jpg", page_number));
        let mut destination = File::create(file_path).unwrap();
        let content = response.bytes().unwrap();
        std::io::copy(&mut content.as_ref(), &mut destination).unwrap();
        return Ok(());
    }

    fn get_issues_list(&self, client: &Client, url: &str) -> Result<Vec<Issue>, SiteDownloaderError> {
        let mut list_of_issues: Vec<Issue> = Vec::new();
        let base_url = "https://".to_string() +  reqwest::Url::parse(url).unwrap().domain().unwrap();
        let link = url; 
        let body = client.get(link).send().unwrap()
            .text().unwrap();
        let document = Html::parse_document(&body);
        let selector = Selector::parse("[data-show-more='#more-chapter']").unwrap();
        let list = document.select(&selector).next();
        match list {
            Some(button) => {
                let link_to_list = base_url.clone() + button.attr("data-path").unwrap();
                let chapters_body = client.get(link_to_list).send().unwrap().text().unwrap();
                let chapters_document = Html::parse_document(&chapters_body);

                let a_selector = Selector::parse("a[href]").unwrap();
                let h5_selector = Selector::parse("h5").unwrap();
                for a_chapter in chapters_document.select(&a_selector) {
                    let href= a_chapter.value().attr("href").unwrap();
                    let issue_link = base_url.clone() + href;
                    let chapter_text= a_chapter.select(&h5_selector)
                        .next().unwrap().text().collect::<Vec<_>>().concat();
                    let chapter_name = chapter_text.lines().nth(1).unwrap().trim();
                    let issue: Issue = Issue { name: chapter_name.to_owned(), link: issue_link };
                    list_of_issues.push(issue);
                }
            },
            None => { //few chapters, no dedicated button
                todo!()
            },
        }
        list_of_issues.reverse();
        return Ok(list_of_issues);
    }

    fn get_comic_name(&self, _client: &Client, url: &str) -> String {
        return url.replace("https://scanita.org/manga/", "").to_string();
    }
}
