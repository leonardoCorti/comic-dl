use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use std::{fs, thread};
use std::io::Write;
use clap::Parser;

use sites::ComicUrl;

mod sites;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Number of threads to use for dowloading
    #[arg(short = 'J', long, default_value = "1")]
    threads: usize,

    /// Number of issues to skip from the start
    #[arg(short= 'S', long, value_name = "SKIP_COUNT", default_value = "0")]
    skip_start: usize,

    /// Number of issues to skip from the last
    #[arg(short= 'L', long, value_name = "SKIP_COUNT", default_value = "0")]
    skip_last: usize,

    /// Download path
    #[arg(short = 'p', long)]
    path: Option<String>,

    /// Download as PDF
    #[arg(long)]
    pdf: bool,

    /// Install to Kobo after download
    #[arg(long)]
    kobo_install: bool,

    /// The link to the comic
    #[arg(required = true)]
    comic_link: String,

    /// interactive mode (todo!)
    #[arg(short= 'I', long)]
    interactive: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let url = args.comic_link;
    if !is_link(&url) {
        eprintln!("the link provided is not a valid url");
        std::process::exit(1);
    }

    if args.interactive {
        todo!();
    }

    if args.kobo_install {
        generate_install(url)?;
        println!("copy the file in the install directory to the kobo");
        return Ok(());
    }
    
    let mut comicdwl = sites::ComicUrl::new(&url).expect("website not supported");

    if args.skip_start > 0 {
        comicdwl.change_skip_first(args.skip_start);
    }

    if args.skip_last > 0 {
        comicdwl.change_skip_lasts(args.skip_last);
    }

    if let Some(ref new_path) = args.path {
        comicdwl.change_path(&new_path)?;
    }

    if args.pdf {
        comicdwl.change_format(sites::OutputFormats::Pdf);
    }

    match args.threads {
        0 => {
            comicdwl.download_all()?
        },
        number => {
            multithread_download(number, comicdwl)?;
        }
    }

    return Ok(());
}

fn multithread_download(
    jobs_quantity: usize,
    comicdwl: ComicUrl,
) -> Result<(), Box<dyn Error>> {
    let issue_list = comicdwl.get_issues_list()?;
    println!("starting download with {jobs_quantity} threads" );
    let comicdwl_arc = Arc::new(comicdwl);
    let mut handles: VecDeque<thread::JoinHandle<()>> = VecDeque::new();
    for issue in issue_list {
        if handles.len() == jobs_quantity {
            handles.pop_front().unwrap().join().unwrap();
        }
        let comicdwl_thread = comicdwl_arc.clone();
        let handle = thread::spawn(move  || {
            comicdwl_thread
                .site_downloader
                .download_issue(
                    &comicdwl_thread.client,
                    &comicdwl_thread.download_path,
                    &issue)
                .expect("couldn't download issue");
            comicdwl_thread.create_volume(
                &issue,
                &comicdwl_thread.download_path
                    .join(issue.name.clone())
            ).expect("couldn't create volume");
        });
        handles.push_back(handle);
    }
    Ok(for handle in handles{
        handle.join().unwrap();
    })
}

fn generate_install(url: String) -> Result<(), Box<dyn Error>>{
    let installation_path = std::path::Path::new("install");
    if !installation_path.exists(){
        fs::create_dir(installation_path)?;
    }
    let kobo_version_link = "https://github.com/leonardoCorti/comic-dl/releases/download/v0.5.0/comic-dl-armv7-linux";
    let script = format!(
r#"#!/bin/sh
cd "$(dirname "$0")"
./comic-dl-armv7-linux {url}"#); 
    let script = script.replace("\\", "/");
    let comic_dw = sites::ComicUrl::new(&url)?;
    let comic_name = &comic_dw.comic_name;

    let mut script_file = File::create(installation_path.join(format!("{comic_name}.sh")))?;
    script_file.write_all(script.as_bytes())?;

    let list_of_file: Vec<String> = installation_path.read_dir().unwrap().into_iter()
        .map(|e| e.unwrap().path()
            .file_name().unwrap()
            .to_str().unwrap()
            .to_string())
        .filter(|e| e.ends_with("sh") && e != "update.sh")
        .collect();

    if list_of_file.len() > 1 {
        let scripts: String = list_of_file.iter().fold("#!/bin/sh \ncd \"$(dirname \"$0\")\"\n".to_string(), |a,b| a + "./" + b + "\n");
        let mut download_all = File::create(installation_path.join("download_all.sh"))?;
        download_all.write_all(scripts.as_bytes())?;
    }

    let update_script = include_str!("./assets/update.sh");
    let mut update_script_file = File::create(installation_path.join("update.sh"))?;
    update_script_file.write_all(update_script.as_bytes())?;

    match reqwest::blocking::Client::new().get(kobo_version_link).send(){
        Ok(program_download) => {
            let progam = program_download.bytes()?;
            if is_elf(&progam[..4].try_into()?) {
                let mut program_file = File::create(installation_path.join("comic-dl-armv7-linux"))?;
                program_file.write_all(&progam)?;
            } else {
                println!("couldn't download the kobo version of comic-dl, donwload it manually");
            }
        }
        Err(_) => {
            println!("couldn't download the kobo version of comic-dl, donwload it manually");
        } ,
    };
    return Ok(());
}

fn is_elf(first_byes: &[u8;4]) -> bool {
    let magic_number = [0x7F, b'E', b'L', b'F'];
    return *first_byes == magic_number;
}

fn is_link(e: &String) -> bool {
    return e.starts_with("https://") || e.starts_with("http://") ;
}
