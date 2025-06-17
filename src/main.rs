use std::time::Instant;
use std::io::{self, BufRead, Write};
use std::net::TcpStream;
use std::time::Duration;
use threadpool::ThreadPool;
use clap::Parser;

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
        println!("YOu can't use -n and -y at the same time");
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

    if let Some(ip_flag) = args.ip_flag {
        println!("Using flag deffined IP");
        ip_in = ip_flag.to_string();
    } else {
        print!("Enter IP or range: ");
        io::stdout().flush().unwrap();
        handle.read_line(&mut ip_in).expect("Failed to read line");
    }

    let ip_in = ip_in.trim();
    let ip_list = parse_ip_range(ip_in);
    let ip_num = ip_list.len();

    if let Some(port_flag) = args.port_flag {
        println!("Using flag deffined port");
        port_in = port_flag.to_string();
    } else {
        print!("Enter Port or range: ");
        io::stdout().flush().unwrap();
        handle.read_line(&mut port_in).expect("Failed to read line");
    }

    let port_in = port_in.trim();
    let port_list = parse_port_range(port_in);
    let port_num = port_list.len();

    if let Some(duration_flag) = args.duration_flag {
        println!("Using flag deffined duration");
        duration_in = duration_flag.to_string();
    } else {
        print!("Enter duration: ");
        io::stdout().flush().unwrap();
        handle.read_line(&mut duration_in).expect("Failed to read line");
    }

    let duration: u64 = duration_in.trim().parse().expect("Failed to parse duration from string to intiger");


    if args.yes {
        display = true;
        println!("\nScanning {} ports on {} IP addresses", port_num, ip_num);
    } else if args.no {
        display = false;
    } else {
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
    }

    let now = Instant::now();

    println!();


    for ip in &ip_list {
        for port in port_list.clone() {
            let tx = tx.clone();
            let ip = ip.clone();

            pool.execute(move || {
                let result = is_open(&ip, port, duration);

                if !display {        
                    let status = if result {
                        format!("Port {} on IP {} is \x1b[1;32mOPEN\x1b[0m", ip, port)
                    } else {
                      format!("Port {} on IP {} is \x1b[1;31mCLOSED\x1b[0m", ip, port)
                    };
                    println!("Status: {}", status);
                } else if display && result {
                    let status = if result {
                        format!("Port {} on IP {} is \x1b[1;32mOPEN\x1b[0m", ip, port)
                    } else {
                      format!("Port {} on IP {} is \x1b[1;31mCLOSED\x1b[0m", ip, port)
                    };

                    println!("Status: {}", status);
                }

                tx.send(()).unwrap();
            });
        }
    }

    for _ in 0..(ip_list.len() * port_list.len())  {
        rx.recv().unwrap();
    }
    
    let elapsed = now.elapsed();
    println!("Scanned {} ports on {} IP addresses in {:.2?}", port_num, ip_num, elapsed);
}
