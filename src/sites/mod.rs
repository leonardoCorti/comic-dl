use std::{fmt::Debug, path::{Path, PathBuf}};

use reqwest::blocking::Client;

pub mod readcomic_me;
pub mod zerocalcare_net;
pub mod scanita_org;

#[derive(Debug)]
pub enum SiteDownloaderError{
    ParsingError,
    NotFound,
    FileSystemError,
}

impl std::fmt::Display for SiteDownloaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error downloading")
    }
}

impl std::error::Error for SiteDownloaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }

}

#[derive(Debug)]
pub struct Issue{
    name: String,
    link: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ComicUrl<T: ComicDownloader>{
    url: String,
    client: Client,
    download_path: PathBuf,
    comic_name: String,
    site_downloader: T,
}

#[allow(dead_code)]
impl<T: ComicDownloader> ComicUrl<T> {
    fn download_all(&self) -> Result<(), SiteDownloaderError> {
        let issues = T::get_issues_list()?;
        issues.iter().for_each(|e| T::download_issue(&e).unwrap());
        return Ok(());
    }
}

pub trait ComicDownloader: Send + Sync + Debug {
    fn download_issue(issue: &Issue) -> Result<(), SiteDownloaderError>;
    fn download_page(link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError>;
    fn get_issues_list() -> Result<Vec<Issue>, SiteDownloaderError>;
    fn create_cbz(issue_name: &Issue, issue_path: PathBuf) -> Result<(), SiteDownloaderError>;
    fn change_path(new_path: &str) -> Result<(), SiteDownloaderError>;
    fn get_comic_name(url: &str) -> &str;
}

pub trait SiteDownloader: Send + Sync + Debug {
    fn download_issue(&self, issue: &Issue) -> Result<(), SiteDownloaderError>;
    fn download_page(&self, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError>;
    fn get_issues_list(&self) -> Result<Vec<Issue>, SiteDownloaderError>;
    fn create_cbz(&self, issue_name: &Issue, issue_path: PathBuf) -> Result<(), SiteDownloaderError>;
    fn change_path(&mut self, new_path: &str) -> Result<(), SiteDownloaderError>;
    fn get_comic_name(&self) -> &str;
    fn download_all(&self) -> Result<(), SiteDownloaderError> {
        let issues = self.get_issues_list()?;
        issues.iter().for_each(|e| self.download_issue(&e).unwrap());
        return Ok(());
    }
}

pub fn new_downloader(url: &str) -> Result<Box<dyn SiteDownloader>, SiteDownloaderError> {
    match reqwest::Url::parse(url){
        Ok(parsed_url) => {
            match parsed_url.domain().unwrap() {
                "readcomic.me" => { return Ok(Box::new(readcomic_me::ReadcomicMe::new(url)))},
                "www.zerocalcare.net" => { return Ok(Box::new(zerocalcare_net::ZerocalcareNet::new(url)))},
                "scanita.org" => { return Ok(Box::new(scanita_org::ScanitaOrg::new(url)))},
                _ => {return Err(SiteDownloaderError::ParsingError)} 
            }
        }, 
        Err(_) => {return Err(SiteDownloaderError::ParsingError)},
    }
}
