use std::collections::VecDeque;
use std::{env, thread};
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
        println!("insert link to comic: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("couldn't read from terminal");
        url = input.trim().to_string();
    }

    let comicdwl = sites::identify_website(&url)?;
    let issue_list = comicdwl.get_issues_list(&url)?;

    let number_of_jobs = args.iter().filter(|e| e.starts_with("-J")).next();
    match number_of_jobs{
        Some(jobs_argument) => {
            let jobs_quantity: usize = jobs_argument.replace("-J", "").parse()?;
            println!("starting download with {jobs_quantity} threads" );
            let mut handles: VecDeque<thread::JoinHandle<()>> = VecDeque::new();
            for issue in issue_list {
                if handles.len() == jobs_quantity {
                    if let Some(handle) = handles.pop_front() {
                        handle.join().unwrap();
                    }
                }
                let copied_url = url.to_string().clone();
                let handle = thread::spawn(move  || {
                    let comicdwl = sites::identify_website(&copied_url).unwrap();
                    comicdwl.download_issue(&issue).unwrap();
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

fn is_link(e: &String) -> bool {
    return e.starts_with("https://") || e.starts_with("http://") ;
}

fn print_help() {
    println!(
r#"Usage: comic-dl [-J<number of threads>] [link to the comic]
Download a comic in the current directory.
will create a directory named after the comic and each chapter will have
a cbz file named <comic name-chapter name>.cbz

options:
   -J<number of threads>    multithreading, one chapter per thread"#);
}
