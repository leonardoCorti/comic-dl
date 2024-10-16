use std::{fmt::Debug, fs::{self, File}, io, path::{Path, PathBuf}, str::FromStr};

#[cfg(feature = "pdf")]
use std::io::{Cursor, Read};

#[cfg(feature = "pdf")]
extern crate printpdf;
#[cfg(feature = "pdf")]
extern crate image as img;

#[cfg(feature = "pdf")]
use printpdf::*;

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
    ImageError,
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

#[derive(Debug, Clone)]
pub struct Issue{
    pub name: String,
    pub link: String,
}

#[allow(dead_code)]
pub enum OutputFormats{
    Pdf,
    Cbz,
}

impl OutputFormats {
    fn format_string(&self) -> &str {
        match self{
            OutputFormats::Pdf => "pdf",
            OutputFormats::Cbz => "cbz",
        }
    
    }
}

#[allow(dead_code)]
pub struct ComicUrl{
    pub url: String,
    pub client: Client,
    pub download_path: PathBuf,
    pub comic_name: String,
    pub format: OutputFormats,
    pub site_downloader: Box<dyn ComicDownloader>,
    pub skip_first: usize,
    pub skip_last: usize,
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
        let skip_first = 0;
        let skip_last = 0;

        return Ok(ComicUrl{ 
            url,
            client,
            download_path, 
            comic_name,
            format,
            site_downloader,
            skip_first,
            skip_last, });
    }

    pub fn download_all(&self) -> Result<(), SiteDownloaderError> {
        let issues = self.get_issues_list()?;
        issues.iter().for_each(|e| {
            self.site_downloader.download_issue(&self.client, &self.download_path, &e).unwrap();
            self.create_volume(e, &self.download_path.join(e.name.clone())).expect("cannot create volume");
        });
        return Ok(());
    }

    pub fn create_volume(&self, issue_name: &Issue, issue_path: &PathBuf) -> Result<(), SiteDownloaderError> {
        let out_filename = format!("{}-{}.{}", self.comic_name, issue_name.name, self.format.format_string());
        let files: Vec<_> = fs::read_dir(&issue_path).expect("could not read files")
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| entry.path().to_str().map(|s| s.to_string()))
            .collect();
        let out_path = self.download_path.join(out_filename);
        match self.format{
            OutputFormats::Pdf => {
                #[allow(unused_assignments, unused_mut)]
                let mut result: Option<()> = None;
                #[cfg(feature = "pdf")] {
                    let doc =  PdfDocument::empty(&self.comic_name);
                    for filename in files {
                        let image = read_image(filename)?;
                        let w = image.image.width.0 as f32;
                        let h = image.image.height.0 as f32;
                        let w_mm = w/300.0 * 25.4;
                        let h_mm = h/300.0 * 25.4;

                        let (page1, layer1) = doc.add_page(Mm(w_mm), Mm(h_mm), "Layer 1");
                        let current_layer = doc.get_page(page1).get_layer(layer1);
                        image.add_to_layer(current_layer.clone(), ImageTransform::default());
                    }
                    doc.save(&mut io::BufWriter::new(File::create(out_path).unwrap()))
                        .expect("error creating pdf");
                    result = Some(());
                }
                if result.is_none() {
                    println!("pdf feature is not enabled");
                    return Err(SiteDownloaderError::ImageError);
                }
            },
            OutputFormats::Cbz => {
                let file = File::create(&out_path).expect("error creating output file");
                let mut zip = zip::ZipWriter::new(file);
                let options = zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Deflated);
                for filename in files {
                    let name = Path::new(&filename).file_name().unwrap().to_str().unwrap();
                    let mut f = File::open(&filename).expect("error creating cbz");
                    zip.start_file(name, options).expect("error creating cbz");
                    io::copy(&mut f, &mut zip).expect("error creating cbz");
                }
                zip.finish().expect("error creating cbz");
            },
        }
        fs::remove_dir_all(&issue_path).expect("couldn't clean source directory");
        return Ok(());
    }

    pub fn change_path(&mut self, new_path_str: &str) -> Result<(), SiteDownloaderError> {
        let new_path = Path::new(new_path_str);
        if !new_path.exists() { return Err(SiteDownloaderError::FileSystemError) };
        self.download_path = new_path.to_path_buf();
        return Ok(());
    }

    pub fn change_format(&mut self, new_format: OutputFormats) {
        self.format = new_format;
    }

    pub fn change_skip_first(&mut self, skip: usize) {
        self.skip_first = skip;
    }

    pub fn change_skip_lasts(&mut self, skip: usize) {
        self.skip_last = skip;
    }

    pub fn get_issues_list(&self) -> Result<Vec<Issue>, SiteDownloaderError> {
        let result: Vec<Issue> = self.site_downloader.get_issues_list(&self.client, &self.url)?;
        let total_issues = result.len();

        if total_issues == 0 {
            return Ok(vec![]);
        }
        let start = self.skip_first.min(total_issues); 
        let end = total_issues.saturating_sub(self.skip_last); 
        if start >= end {
            return Ok(vec![]);
        }
        let sliced_result = result[start..end].to_vec();

        Ok(sliced_result)
    }
}

#[cfg(feature = "pdf")]
fn read_image(filename: String) -> Result<Image,SiteDownloaderError> {
    let mut image_file = File::open(filename).expect("error opening file");
    let mut buffer = Vec::new();
    image_file.read_to_end(&mut buffer).unwrap();
    let format = img::guess_format(&buffer).unwrap();
    let cursor = Cursor::new(buffer);
    let image = match format{
        ::image::ImageFormat::Png => {
            Image::try_from( 
                image_crate::codecs::png::PngDecoder::new(cursor)
                    .expect("couldn't decode image"))
                .unwrap()
        },
        ::image::ImageFormat::Jpeg => {
            Image::try_from( 
                image_crate::codecs::jpeg::JpegDecoder::new(cursor)
                    .expect("couldn't decode image"))
                .unwrap()
        },
        ::image::ImageFormat::WebP => {
            let webp_image = img::load(cursor, img::ImageFormat::WebP).unwrap();
            let mut png_bytes: Vec<u8> = Vec::new();
            webp_image.write_to(
                &mut Cursor::new(&mut png_bytes),
                img::ImageFormat::Png)
                .unwrap();
            let png_cursor = Cursor::new(png_bytes);
            Image::try_from( 
                image_crate::codecs::png::PngDecoder::new(png_cursor)
                    .expect("couldn't decode image"))
                .unwrap()
        },
        _ => return Err(SiteDownloaderError::ImageError),
    };
    Ok(image)
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

pub fn print_supported_websites() -> String {
return r#"
- https://readcomic.me
- https://www.zerocalcare.net
- https://scanita.org
"#.to_string();
}

pub trait ComicDownloader: Send + Sync + Debug {
    fn download_issue(&self, client: &Client, download_path: &PathBuf, issue: &Issue) -> Result<(), SiteDownloaderError>;
    fn download_page(&self, client: &Client, link: &str, issue_path: &Path, page_number: u32) -> Result<(), SiteDownloaderError>;
    fn get_issues_list(&self, client: &Client, url: &str) -> Result<Vec<Issue>, SiteDownloaderError>;
    fn get_comic_name(&self, client: &Client,  url: &str) -> String;
}
