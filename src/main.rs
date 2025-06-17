use std::time::Instant;
use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;
use clap::Parser;
use json::JsonValue;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short = 'i', long = "ip")]
    ip_flag: Option<String>,

    #[arg(short = 'p', long = "port")]
    port_flag: Option<String>,

    #[arg(short = 'd', long = "duration")]
    duration_flag: Option<u64>,

    #[arg(short = 'y', long = "yes")]
    yes: bool,

    #[arg(short = 'n', long = "no")]
    no: bool,

    #[arg(short = 'j', long = "json")]
    json: bool,
}

fn is_open(ip: &str, port: u16, duration: u64) -> bool {
    let addr = format!("{}:{}", ip, port);
    let timeout = Duration::from_millis(duration);
    TcpStream::connect_timeout(&addr.parse().unwrap(), timeout).is_ok()
}

fn parse_ip_range(input: &str) -> Vec<String> {
    if let Some((base, range)) = input.rsplit_once('.') {
        if let Some((start, end)) = range.split_once('-') {
            let start: u8 = start.parse().unwrap_or(0);
            let end: u8 = end.parse().unwrap_or(0);
            return (start..=end).map(|i| format!("{}.{}", base, i)).collect();
        }
    }
    vec![input.to_string()]
}

fn parse_port_range(input: &str) -> Vec<u16> {
    if let Some((start, end)) = input.split_once('-') {
        let start: u16 = start.parse().unwrap_or(0);
        let end: u16 = end.parse().unwrap_or(0);
        return (start..=end).collect();
    }
    match input.trim().parse() {
        Ok(p) => vec![p],
        Err(_) => vec![],
    }
}

fn main() {

    let args = Args::parse();

    if args.yes && args.no {
        println!("You can't use -n and -y at the same time");
    }

    let n_workers = num_cpus::get();
    let pool = ThreadPool::new(n_workers);

    let (tx, rx): (std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>) = std::sync::mpsc::channel();

    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut ip_in = String::new();
    let mut port_in = String::new();
    let mut duration_in = String::new();
    let mut display_in = String::new();
    let mut display: bool = false;

    if !args.json {
        if let Some(ip_flag) = args.ip_flag {
            println!("Using flag deffined IP");
            ip_in = ip_flag.to_string();
        } else {
            print!("Enter IP or range: ");
            io::stdout().flush().unwrap();
            handle.read_line(&mut ip_in).expect("Failed to read line");
        }
    } else {
        if let Some(ip_flag) = args.ip_flag {
            ip_in = ip_flag.to_string();
        } else {
            eprintln!("[\x1b[1;31m ERROR \x1b[0m] User did not supply the ip address or range as a flag as is nessecary with '--json'");
            std::process::exit(1);
        }
    }

    let ip_in = ip_in.trim();
    let ip_list = parse_ip_range(ip_in);
    let ip_num = ip_list.len();

    if !args.json {
        if let Some(port_flag) = args.port_flag {
            println!("Using flag deffined port");
            port_in = port_flag.to_string();
        } else {
            print!("Enter Port or range: ");
            io::stdout().flush().unwrap();
            handle.read_line(&mut port_in).expect("Failed to read line");
        }
    } else {
        if let Some(port_flag) = args.port_flag {
            port_in = port_flag.to_string();
        } else {
            eprintln!("[\x1b[1;31m ERROR \x1b[0m] User did not supply the port or range as a flag as is nessecary with '--json'");
            std::process::exit(1);
        }
    }

    let port_in = port_in.trim();
    let port_list = parse_port_range(port_in);
    let port_num = port_list.len();

    if !args.json {
        if let Some(duration_flag) = args.duration_flag {
            println!("Using flag deffined duration");
            duration_in = duration_flag.to_string();
        } else {
            print!("Enter duration: ");
            io::stdout().flush().unwrap();
            handle.read_line(&mut duration_in).expect("Failed to read line");
        }
    } else {
        if let Some(duration_flag) = args.duration_flag {
            duration_in = duration_flag.to_string();
        } else {
            eprintln!("[\x1b[1;31m ERROR \x1b[0m] User did not supply the duration as a flag as is nessecary with '--json'");
            std::process::exit(1);
        }
    }

    let duration: u64 = duration_in.trim().parse().expect("Failed to parse duration from string to intiger");

    if args.yes {
        display = true;
        if !args.json {
            println!("\nScanning {} ports on {} IP addresses", port_num, ip_num);
        }
    } else if args.no {
        display = false;
    } else {
        if !args.json {
            if ip_list.len() > 1 || port_list.len() > 1{
                print!("Only display open (y/n): ");
                io::stdout().flush().unwrap();
                handle.read_line(&mut display_in).expect("Failed to read line");
                display = if display_in.trim() == "y" {
                    true
                } else {
                    false
                };

                println!("\nScanning {} ports on {} IP addresses", port_num, ip_num);
            }
        } else {
            eprintln!("[\x1b[1;31m ERROR \x1b[0m] User did not supply yes or no flag for weather or not to ouput only open ports as is nessecary with '--json'");
            std::process::exit(1);
        }
    }

    let now = Instant::now();

    println!();

    let json_results = Arc::new(Mutex::new(Vec::new()));

    for ip in &ip_list {
        for port in port_list.clone() {
            let tx = tx.clone();
            let ip = ip.clone();
            let json_results = Arc::clone(&json_results);

            pool.execute(move || {
                let result = is_open(&ip, port, duration);

                if !display {        
                    if !args.json {
                        let status = if result {
                            format!("Port {} on IP {} is \x1b[1;32mOPEN\x1b[0m", ip, port)
                        } else {
                            format!("Port {} on IP {} is \x1b[1;31mCLOSED\x1b[0m", ip, port)
                        };
                        println!("Status: {}", status);
                    } else {
                        let mut obj = JsonValue::new_object();

                        obj["ip"] = ip.into();
                        obj["port"] = port.into();
                        obj["status"] = result.into();

                        json_results.lock().unwrap().push(obj);
                    }
                } else if display && result {
                    if !args.json {
                        let status = if result {
                            format!("Port {} on IP {} is \x1b[1;32mOPEN\x1b[0m", ip, port)
                        } else {
                          format!("Port {} on IP {} is \x1b[1;31mCLOSED\x1b[0m", ip, port)
                        };

                        println!("Status: {}", status);
                    } else {
                        let mut obj = JsonValue::new_object();

                        obj["ip"] = ip.into();
                        obj["port"] = port.into();
                        obj["status"] = result.into();
                        
                        json_results.lock().unwrap().push(obj);
                    }
                }

                tx.send(()).unwrap();
            });
        }
    }

    pool.join();

    if args.json {
        let array = {
            let locked = json_results.lock().unwrap();
            let mut arr = JsonValue::new_array();
            for obj in locked.iter() {
                arr.push(obj.clone()).unwrap();
            }
            arr
        };

        println!("{}", array.dump());
    }

    for _ in 0..(ip_list.len() * port_list.len())  {
        rx.recv().unwrap();
    }
    
    if !args.json {
        let elapsed = now.elapsed();
        println!("Scanned {} ports on {} IP addresses in {:.2?}", port_num, ip_num, elapsed);
    }
}
