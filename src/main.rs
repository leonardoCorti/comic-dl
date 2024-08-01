use std::collections::VecDeque;
use std::{env, thread};
use std::io::{self, Write};

mod sites;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let url: String;
    if args.len() < 2 {
        println!("insert link to comic: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("couldn't read from terminal");
        url = input.trim().to_string();
    } else {
        url = args[1].clone();
    }

    let comicdwl = sites::identify_website(&url).unwrap();
    let issue_list = comicdwl.get_issues_list(&url)?;

    let args: Vec<_> =  std::env::args().collect();
    let number_of_jobs = args.iter().filter(|e| e.starts_with("-J")).last();
    match number_of_jobs{
        Some(jobs_argument) => {
            let jobs_quantity: usize = jobs_argument.replace("-J", "").parse()?;
            println!("starting download with {jobs_quantity} threads" );
            let mut handles: VecDeque<thread::JoinHandle<()>> = VecDeque::new();
            for _issue in issue_list {
                if handles.len() == jobs_quantity {
                    if let Some(handle) = handles.pop_front() {
                        handle.join().unwrap();
                    }
                }
                let handle = thread::spawn(move  || {
                    //comicdwl.download_issue(&issue).expect("error downloading");
                    todo!();
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
