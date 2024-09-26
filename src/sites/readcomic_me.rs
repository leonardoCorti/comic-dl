use regex::Regex;
use reqwest::blocking::Client;
use scraper::{selectable::Selectable, Html, Selector};
use std::{fs::{self, File}, io, path::{Path, PathBuf}};

use super::*;

#[derive(Debug, Clone)]
pub struct ReadcomicMeStrategy ;

impl ReadcomicMeStrategy{
    fn get_page_with_issues(&self, client: &Client, page_link: String) -> Option<String> {
        let response = client.get(page_link).send().unwrap();
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

#[allow(unused_variables)]
impl ComicDownloader for ReadcomicMeStrategy{
    fn download_issue(&self, client: &Client, download_path: &PathBuf, issue: &Issue) -> Result<(), SiteDownloaderError> {
        if !download_path.exists() {
            if fs::create_dir(download_path).is_err() {
                if !download_path.exists(){
                    return Err(SiteDownloaderError::FileSystemError);
                }
            };
        }
        let issue_link = &issue.link;
        let issue_number = &issue.name;
        println!("Downloading issue number {}", issue_number);
        let issue_path = download_path.join(issue_number);
        let out_filename = format!("{}-{}.cbz", self.get_comic_name(client, &issue_link), issue.name);
        let out_path = download_path.join(&out_filename);
        if Path::new(&out_path).exists() {
            println!("Was already downloaded");
            return Ok(());
        }
        if !issue_path.exists() {
            fs::create_dir(&issue_path).unwrap();
        }

        let page = client.get(issue_link).send().unwrap();
        let page_body = page.text().unwrap();
        let document = Html::parse_document(&page_body);
        let total_pages_selector = Selector::parse("span.total-pages").unwrap();
        let total_pages_text = document.select(&total_pages_selector).next().unwrap().inner_html();
        let second_pattern = Regex::new(r"(\d+)").unwrap();
        let number_of_pages = second_pattern.captures(&total_pages_text).unwrap().get(1).unwrap().as_str().parse::<i32>().unwrap();
        println!("There are {} pages", number_of_pages);

        for n in 0..number_of_pages {
            let page_link = format!("{}/{}", issue_link, n + 1);
            self.download_page(client, &page_link, &issue_path, (n+1).try_into().unwrap()).unwrap();
        }

        return Ok(());
    }

    fn download_page(&self, client: &Client, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError> {
        let page = client.get(link).send().unwrap();
        let page_body = page.text().unwrap();
        let document = Html::parse_document(&page_body);
        let img_selector = Selector::parse("img.single-page").unwrap();
        let page_image_link = document.select(&img_selector).next().unwrap().value().attr("src").unwrap();
        let page_name = format!("{:04}.jpg", page_number);
        let dw_path =  issue_path.join(page_name);
        let mut response = client.get(page_image_link).send().unwrap();
        if response.status().is_success() {
            let mut file = File::create(dw_path).unwrap();
            io::copy(&mut response, &mut file).unwrap();
        } else {
            eprintln!("Couldn't download page");
        }
        return Ok(());
    }

    fn get_issues_list(&self, client: &Client, url: &str) -> Result<Vec<Issue>, SiteDownloaderError> {
        let mut vec = Vec::new();
        let link = url.to_string();
        let mut page_number = 1;
        let mut page_link = link.to_string() + "?page="+&page_number.to_string();
        while let Some(page_with_link) = self.get_page_with_issues(client, page_link) {
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

    fn get_comic_name(&self, _client: &Client,  url: &str) -> String {
        return url.replace("https://readcomic.me/comic/", "");
    }
}
