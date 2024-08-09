use std::{collections::HashSet, fs::{self, File}, io::{self, copy}};

use reqwest::blocking::Client;

use super::*;

#[derive(Debug, Clone)]
pub struct ZerocalcareNet {
    base_url: String,
    comic_url: String,
    client: Client,
    download_path: PathBuf,
    comic_name: String,
}

impl SiteDownloader for ZerocalcareNet {}

impl ZerocalcareNet {
    pub fn new(comic_path: &str) -> Self {
        let base_url = "https://www.zerocalcare.net".to_string();
        let client = Client::new();
        let comic_url = comic_path.replace(&base_url, "");
        let download_path = Path::new(&comic_url.replace("/storie-a-fumetti/", "").strip_suffix("/").unwrap()).to_owned();
        let comic_name = comic_path.replace("https://www.zerocalcare.net/storie-a-fumetti/", "").strip_suffix("/").expect("couldn't find comic name").into();
        Self { base_url, comic_url, client, download_path, comic_name }
    }
}

impl SiteDownloaderFunctions for ZerocalcareNet {
    fn download_issue(&self, issue: &Issue) -> Result<(), SiteDownloaderError> {
        if !self.download_path.exists() {
            if fs::create_dir(&self.download_path).is_err() {
                if !self.download_path.exists(){
                    return Err(SiteDownloaderError::FileSystemError);
                }
            };
        }
        let issue_link = &issue.link;
        let issue_name = &issue.name;
        println!("Downloading {}", self.comic_url);
        let issue_path = self.download_path.join(issue_name);
        let out_filename = format!("{}.cbz", self.comic_name);
        let out_path = self.download_path.join(&out_filename);
        if Path::new(&out_path).exists() {
            println!("Was already downloaded");
            return Ok(());
        }
        fs::create_dir(&issue_path).unwrap();

        let regex_find_pages = regex::Regex::new(r"https://www.zerocalcare.net/wp-content/uploads/\d{4}/\d{2}/(\d+)-(\d+).jpg").unwrap();
        let page = self.client.get(issue_link).send().unwrap();
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

            self.download_page(&page, &issue_path, page_number).unwrap();
        }

        self.create_cbz(issue, issue_path)?;

        return Ok(());
    }

    fn download_page(&self, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError> {
        //println!("downloading page {}, {}, {:04}", link, issue_path.to_str().unwrap(), page_number);
        let response = self.client.get(link).send().expect("can't download a page");
        let file_path = issue_path.join(format!("{:04}.jpg", page_number));
        let mut destination = File::create(file_path).unwrap();
        let content = response.bytes().unwrap();
        copy(&mut content.as_ref(), &mut destination).unwrap();
        return Ok(());
    }

    fn get_issues_list(&self) -> Result<Vec<Issue>, SiteDownloaderError> {
        //there is a single issue for comic
        let name = self.comic_name.clone();
        let link = self.base_url.clone() + self.comic_url.as_str();
        let the_issue: Issue = Issue { name, link };
        let vec = vec![the_issue];
        return Ok(vec);
    }

    fn create_cbz(&self, issue_name: &Issue, issue_path: PathBuf) -> Result<(), SiteDownloaderError> {
        let out_filename = format!("{}.cbz", issue_name.name);
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
