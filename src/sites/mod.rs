use std::{fmt::Debug, fs::{self, File}, io, path::{Path, PathBuf}, str::FromStr};

use readcomic_me::ReadcomicMeStrategy;
use reqwest::blocking::Client;
use scanita_org::ScanitaOrgStrategy;
use zerocalcare_net::ZerocalcareNetStrategy;

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
    pub name: String,
    pub link: String,
}

#[allow(dead_code)]
pub enum OutputFormats{
    Pdf,
    Cbz,
}

#[allow(dead_code)]
pub struct ComicUrl{
    pub url: String,
    pub client: Client,
    pub download_path: PathBuf,
    pub comic_name: String,
    pub format: OutputFormats,
    pub site_downloader: Box<dyn ComicDownloader>,
}

#[allow(dead_code)]
impl ComicUrl {

    pub fn new(url: &str) -> Result<ComicUrl, SiteDownloaderError> {
        let url = url
            .to_string();
        let client = reqwest::blocking::Client::new();
        let site_downloader = identify_website(&url)?;
        let comic_name = site_downloader.get_comic_name(&client,&url)
            .to_string();
        let download_path = PathBuf::from_str(&comic_name)
            .expect("cannot create download folder");
        let format = OutputFormats::Cbz;

        return Ok(ComicUrl{ 
            url,
            client,
            download_path, 
            comic_name,
            format,
            site_downloader });
    }


    pub fn download_all(&self) -> Result<(), SiteDownloaderError> {
        let issues = self.site_downloader.get_issues_list(&self.client, &self.url)?;
        issues.iter().for_each(|e| {
            self.site_downloader.download_issue(&self.client, &self.download_path, &e).unwrap();
            self.create_volume(e, &self.download_path.join(e.name.clone())).expect("cannot create volume");
        });
        return Ok(());
    }

    pub fn create_volume(&self, issue_name: &Issue, issue_path: &PathBuf) -> Result<(), SiteDownloaderError> {
        match self.format{
            OutputFormats::Pdf => {
                todo!()
            },
            OutputFormats::Cbz => {
                let out_filename = format!("{}-{}.cbz", self.comic_name, issue_name.name);
                let out_path = self.download_path.join(out_filename);
                let file = File::create(&out_path).expect("error creating cbz");
                let mut zip = zip::ZipWriter::new(file);
                let options = zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);
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
            },
        }
        return Ok(());
    }

    pub fn change_path(&mut self, new_path_str: &str) -> Result<(), SiteDownloaderError> {
        let new_path = Path::new(new_path_str);
        if !new_path.exists() { return Err(SiteDownloaderError::FileSystemError) };
        self.download_path = new_path.to_path_buf();
        return Ok(());
    }

    pub fn get_issues_list(&self) -> Result<Vec<Issue>, SiteDownloaderError> {
        self.site_downloader.get_issues_list(&self.client, &self.url)
    }
}

fn identify_website(url: &str) -> Result<Box<dyn ComicDownloader>, SiteDownloaderError> {

    match reqwest::Url::parse(url){
        Ok(parsed_url) => {
            match parsed_url.domain().unwrap() {
                "readcomic.me" => { return Ok(Box::new(ReadcomicMeStrategy))},
                "www.zerocalcare.net" => { return Ok(Box::new(ZerocalcareNetStrategy))},
                "scanita.org" => { return Ok(Box::new(ScanitaOrgStrategy))},
                _ => {return Err(SiteDownloaderError::ParsingError)} 
            }
        }, 
        Err(_) => {return Err(SiteDownloaderError::ParsingError)},
    }
}

pub trait ComicDownloader: Send + Sync + Debug {
    fn download_issue(&self, client: &Client, download_path: &PathBuf, issue: &Issue) -> Result<(), SiteDownloaderError>;
    fn download_page(&self, client: &Client, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError>;
    fn get_issues_list(&self, client: &Client, url: &str) -> Result<Vec<Issue>, SiteDownloaderError>;
    fn get_comic_name(&self, client: &Client,  url: &str) -> String;
}
