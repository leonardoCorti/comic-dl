use std::collections::VecDeque;
use std::fs::File;
use std::sync::Arc;
use std::{env, fs, thread};
use std::io::{self, Write};

mod sites;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.iter().filter(|e| *e=="-h" || *e=="--help").count() != 0 {
        print_help();
        return Ok(());
    }
    let url: String;
    if let Some(link) = args.iter().filter(|e| is_link(e)).next() {
        url = link.to_string();
    } else {
        println!("select function:\n 1)download\n 2)create kobo install\n");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("couldn't read from terminal");
        let function = input.trim().to_string();
        match function.as_str() {
            "1" => { /*continue*/}
            "2" =>{
                println!("insert link to comic: ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("couldn't read from terminal");
                url = input.trim().to_string();
                println!("insert path of the e-reader: ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("couldn't read from terminal");
                let installation_path = input.trim().to_string();
                generate_install(&installation_path, url)?;
                println!("copy the file in the install directory to {installation_path}");
                return Ok(());
            }
            _ => {
                println!("invalid option");
                return Ok(());
            }

        }
        println!("insert link to comic: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("couldn't read from terminal");
        url = input.trim().to_string();
    }

    if let Some(install_flag) = args.iter().position(|e| e == "--kobo-install"){
        if let Some(install_position) = args.iter().nth(install_flag +1){
            generate_install(install_position, url)?;
            println!("copy the file in the install directory to {install_position}");
            return Ok(());
        } else {
            println!("no path detected after --kobo-install flag");
            return Ok(());
        }
    }

    let mut custom_path: Option<String> = Option::None;
    if let Some(p_flag_position) = args.iter().position(|e| e == "-p"){
        if let Some(new_path) = args.iter().nth(p_flag_position +1){
            custom_path = Some(new_path.to_string());
        } else {
            println!("no path detected after -p flag");
            return Ok(());
        }
    }

    let mut comicdwl = sites::new_downloader(&url)?;
    if let Some(ref new_path) = custom_path {
        comicdwl.change_path(&new_path)?;
    }
    let issue_list = comicdwl.get_issues_list(&url)?;

    let number_of_jobs = args.iter().filter(|e| e.starts_with("-J")).next();
    match number_of_jobs{
        Some(jobs_argument) => {
            let jobs_quantity: usize = jobs_argument.replace("-J", "").parse()?;
            println!("starting download with {jobs_quantity} threads" );
            let comicdwl_arc = Arc::new(comicdwl);
            let mut handles: VecDeque<thread::JoinHandle<()>> = VecDeque::new();
            for issue in issue_list {
                if handles.len() == jobs_quantity {
                    if let Some(handle) = handles.pop_front() {
                        handle.join().unwrap();
                    }
                }
                let my_comicdwl = comicdwl_arc.clone();
                let handle = thread::spawn(move  || {
                    my_comicdwl.download_issue(&issue).unwrap();
                });

                handles.push_back(handle);
            }

            for handle in handles{
                handle.join().unwrap();
            }

        },
        None => {
            for issue in issue_list {
                comicdwl.download_issue(&issue)?;
            }
        },
    }

    return Ok(());
}

fn generate_install(install_position: &str, url: String) -> Result<(), Box<dyn std::error::Error>>{
    let installation_path = std::path::Path::new("install");
    if !installation_path.exists(){
        fs::create_dir(installation_path)?;
    }
    let kobo_version_link = "https://github.com/leonardoCorti/comic-dl/releases/download/v0.3.5/comic-dl-armv7-linux";
    let script = format!(
r#"#!/bin/sh
cd /mnt/onboard/{install_position}
./comic-dl-armv7-linux {url}"#); 
    let script = script.replace("\\", "/");
    let comic_dw = sites::new_downloader(&url)?;
    let comic_name = comic_dw.get_comic_name();

    let mut script_file = File::create(installation_path.join(format!("{comic_name}.sh")))?;
    script_file.write_all(script.as_bytes())?;

    let mut program_file = File::create(installation_path.join("comic-dl-armv7-linux"))?;
    let progam = reqwest::blocking::Client::new().get(kobo_version_link).send()?.bytes()?;
    program_file.write_all(&progam)?;

    let list_of_file: Vec<String> = installation_path.read_dir().unwrap().into_iter()
        .map(|e| e.unwrap().path()
            .file_name().unwrap()
            .to_str().unwrap()
            .to_string())
        .filter(|e| e.ends_with("sh"))
        .collect();

    if list_of_file.len() > 1 {
        let scripts: String = list_of_file.iter().fold("#!/bin/sh \n".to_string(), |a,b| a + "./" + b + "\n");
        let mut download_all = File::create(installation_path.join("download_all.sh"))?;
        download_all.write_all(scripts.as_bytes())?;
    }
    return Ok(());
}

fn is_link(e: &String) -> bool {
    return e.starts_with("https://") || e.starts_with("http://") ;
}

fn print_help() {
    println!(
r#"Usage: comic-dl [-J<number of threads>] [-p <download path>] [--kobo-install <path>] [link to the comic]
Download a comic in the current directory.
will create a directory named after the comic and each chapter will have
a cbz file named <comic name-chapter name>.cbz
The path for the --kobo-install option should be the path where you want to download the comic on the kobo, exclude the drive letter on windows, for example G:\comics\spiderman should just be --kobo-install comics\spiderman

options:
   -J<number of threads>    multithreading, one chapter per thread
   -p <download path>       custom download path
   --kobo-install <path>    setup the script to use on kobo"#);
}
