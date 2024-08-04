#![allow(dead_code)]

use std::{fs::{self, File}, path::PathBuf};

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
        let issue_path = self.download_path.join(issue.name.clone());
        if !issue_path.exists(){
            fs::create_dir(&issue_path).unwrap();
        }
        let mut page_number: u32 = 1;
        loop {
            let link = issue.link.clone() + "/" + &page_number.to_string();
            let request = self.client.get(link).send().expect("couldn't donwload page");
            if request.status() == reqwest::StatusCode::FOUND {break;}
            let page =  request.text().unwrap();
            let document = Html::parse_document(&page);
            let page_selector = Selector::parse(".book-page").unwrap();
            let img_selector = Selector::parse("img").unwrap();
            let page_div = document.select(&page_selector).next();
            if page_div.is_none() {break;}
            let page_img = page_div.unwrap().select(&img_selector).next().unwrap();
            let page_img_link = page_img.attr("src").unwrap();
            self.download_page(page_img_link, &issue_path , page_number)?;
            page_number += 1;
        }

        self.create_cbz(issue, issue_path)?;
        return Ok(());
    }

    fn download_page(&self, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError> {
        let response = self.client.get(link).send().expect("can't download a page");
        let file_path = issue_path.join(format!("{:04}.jpg", page_number));
        let mut destination = File::create(file_path).unwrap();
        let content = response.bytes().unwrap();
        std::io::copy(&mut content.as_ref(), &mut destination).unwrap();
        return Ok(());
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
                    let chapter_name = chapter_text.lines().nth(1).unwrap().trim();
                    println!("downloading {chapter_name}");

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
            std::io::copy(&mut f, &mut zip).expect("error creating cbz");
        }
        zip.finish().expect("error creating cbz");
        fs::remove_dir_all(&issue_path).expect("couldn't clean source directory");
        return Ok(());
    }
}

