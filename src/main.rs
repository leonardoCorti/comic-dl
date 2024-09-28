use std::collections::VecDeque;
use std::error::Error;
use std::fs::File;
use std::sync::Arc;
use std::{env, fs, thread};
use std::io::{self, Write};

use sites::ComicUrl;

mod sites;

fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();
    // --help check
    if args.iter().filter(|e| *e=="-h" || *e=="--help").count() != 0 {
        print_help();
        return Ok(());
    }
    // url check
    let url: String;
    if let Some(link) = args.iter().filter(|e| is_link(e)).next() {
        url = link.to_string();
    } else {
        println!("insert link to comic: ");
        url = read_from_terminal().trim().to_string();
        println!("select function:\n 1)download\n 2)create kobo install\n");
        match read_from_terminal().trim().to_string().as_str() {
            "1" => {/*continue*/}
            "2" =>{
                interactive_kobo_installation(url)?;
                return Ok(());
            }
            _ => {
                println!("invalid option");
                return Ok(());
            }
        }
    }
    // --kobo-install check
    if let Some(_install_flag) = args.iter().position(|e| e == "--kobo-install"){
        generate_install(url)?;
        println!("copy the file in the install directory to the kobo");
        return Ok(());
    }
    // -p check
    let mut custom_path: Option<String> = Option::None;
    if let Some(p_flag_position) = args.iter().position(|e| e == "-p"){
        if let Some(new_path) = args.iter().nth(p_flag_position +1){
            custom_path = Some(new_path.to_string());
        } else {
            println!("no path detected after -p flag");
            return Ok(());
        }
    }
    // -J check
    let number_of_jobs = args.iter().filter(|e| e.starts_with("-J")).next();
    // --pdf check
    let is_pdf = args.contains(&"--pdf".to_string());

    //start program
    //let mut comicdwl = sites::new_downloader(&url)?;
    let mut comicdwl = sites::ComicUrl::new(&url)?;

    if let Some(ref new_path) = custom_path {
        comicdwl.change_path(&new_path)?;
    }
    if is_pdf {
        comicdwl.change_format(sites::OutputFormats::Pdf);
    }

    match number_of_jobs{
        Some(jobs_argument) => {
            let jobs_quantity: usize = jobs_argument.replace("-J", "").parse()?;
            multithread_download(jobs_quantity, comicdwl)?;
        },
        None => {
            comicdwl.download_all()?;
        },
    }

    return Ok(());
}

fn interactive_kobo_installation(url: String) -> Result<(), Box<dyn Error>> {
    generate_install(url)?;
    println!("copy the file in the install directory to the kobo");
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

fn read_from_terminal() -> String {
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("couldn't read from terminal");
    return input;
}

fn generate_install(url: String) -> Result<(), Box<dyn Error>>{
    let installation_path = std::path::Path::new("install");
    if !installation_path.exists(){
        fs::create_dir(installation_path)?;
    }
    let kobo_version_link = "https://github.com/leonardoCorti/comic-dl/releases/download/v0.4.0/comic-dl-armv7-linux";
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

fn print_help() {
    println!(
r#"Usage: comic-dl [-J<number of threads>] [-p <download path>] [--pdf] [--kobo-install] [link to the comic]
Download a comic in the current directory.
will create a directory named after the comic and each chapter will have
a cbz file named <comic name-chapter name>.cbz

options:
   -J<number of threads>    multithreading, one chapter per thread
   -p <download path>       custom download path
   --pdf                    pdf output
   --kobo-install           setup the script to use on kobo"#);
}
