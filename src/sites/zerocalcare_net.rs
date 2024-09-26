use std::{collections::HashSet, fs::{self, File}, io::copy};

use reqwest::blocking::Client;

use super::*;


#[derive(Debug, Clone)]
pub struct ZerocalcareNetStrategy;

impl ComicDownloader for ZerocalcareNetStrategy{
    fn download_issue(&self, client: &Client, download_path: &PathBuf, issue: &Issue) -> Result<(), SiteDownloaderError> {
        if !download_path.exists() {
            if fs::create_dir(&download_path).is_err() {
                if !download_path.exists(){
                    return Err(SiteDownloaderError::FileSystemError);
                }
            };
        }
        let issue_link = &issue.link;
        let issue_name = &issue.name;
        println!("Downloading {}", issue.name);
        let issue_path = download_path.join(issue_name);
        let out_filename = format!("{}.cbz", issue_name);
        let out_path = download_path.join(&out_filename);
        if Path::new(&out_path).exists() {
            println!("Was already downloaded");
            return Ok(());
        }
        fs::create_dir(&issue_path).unwrap();

        let regex_find_pages = regex::Regex::new(r"https://www.zerocalcare.net/wp-content/uploads/\d{4}/\d{2}/(\d+)-(\d+).jpg").unwrap();
        let page = client.get(issue_link).send().unwrap();
        let page_body = page.text().unwrap();
        let mut pages = HashSet::new();
        for line in page_body.lines(){
            for mat in regex_find_pages.find_iter(&line) {
                pages.insert(mat.as_str().to_string());
            }
        }

        for page in pages {
            let regex_find_page_number = regex::Regex::new(r"https://www\.zerocalcare\.net/wp-content/uploads/\d{4}/\d{2}/(\d+)-\d+\.jpg").unwrap();
            let page_number = regex_find_page_number.captures_iter(&page).nth(0).unwrap().get(1).unwrap().as_str().parse::<u32>().unwrap();
            //println!("{page} cap {:04}", page_number);

            self.download_page(&client, &page, &issue_path, page_number).unwrap();
        }
        return Ok(());
    }

    fn download_page(&self, client: &Client, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError> {
        //println!("downloading page {}, {}, {:04}", link, issue_path.to_str().unwrap(), page_number);
        let response = client.get(link).send().expect("can't download a page");
        let file_path = issue_path.join(format!("{:04}.jpg", page_number));
        let mut destination = File::create(file_path).unwrap();
        let content = response.bytes().unwrap();
        copy(&mut content.as_ref(), &mut destination).unwrap();
        return Ok(());
    }

    fn get_issues_list(&self, client: &Client, url: &str) -> Result<Vec<Issue>, SiteDownloaderError> {
        //there is a single issue for comic
        let name = self.get_comic_name(client, url);
        let link = url.to_string();
        let the_issue: Issue = Issue { name, link };
        let vec = vec![the_issue];
        return Ok(vec);
    }

    fn get_comic_name(&self, _client: &Client,  url: &str) -> String {
        return url
            .replace("https://www.zerocalcare.net/storie-a-fumetti/", "")
            .strip_suffix("/")
            .expect("couldn't find comic name").to_string();
    }
}
