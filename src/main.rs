use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;
use std::net::{IpAddr, ToSocketAddrs};
use std::error::Error;
use clap::Parser;

// https://docs.rs/clap/latest/clap/struct.Arg.html#method.value_delimiter
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(num_args = 0..)]
    hostname: Vec<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    for hostname in args.hostname {
        match get_host_by_name(&hostname) {
            Ok(ip) => {
                println!("{}: {}", hostname, ip);
                let hostname = hostname.trim_start_matches("csc");
                update_hostname(&ip, &hostname)?;
                let hostname = format!("csc{}", hostname);
                print_hostname_ip(&ip, &hostname)?;
                restart_printer(&ip)?;
            },
            Err(e) => eprintln!("Error resolving {}: {}", hostname, e),
        }
    }
    Ok(())
}

fn restart_printer(ip: &IpAddr) -> std::io::Result<()> {
    println!("Restarting printer in 5 seconds...");
    std::thread::sleep(Duration::from_secs(5));
    let mut stream = TcpStream::connect((ip.to_string().as_str(), 9100))?;
    let data = "~JR";
    stream.write_all(data.as_bytes())?;
    println!("Sent restart command to {}", ip);
    Ok(())
}

fn print_hostname_ip(ip: &IpAddr, hostname: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect((ip.to_string().as_str(), 9100))?;

    let data = format!("^XA^A0N,32,32^FO32,32^FD{}^FS^A0N,32,32^FO32,128^FD{}^FS^XZ", hostname, ip);
    stream.write_all(data.as_bytes())?;

    Ok(())
}

fn update_hostname(ip: &IpAddr, hostname: &str) -> std::io::Result<()> {
    let mut stream = TcpStream::connect((ip.to_string().as_str(), 9100))?;
    
    let hostname = hostname.trim_start_matches("csc");

    let data = format!("^XA^KNcsc{}^JUS^XZ", hostname);
    stream.write_all(data.as_bytes())?;

    println!("Updated hostname to csc{} on {}", hostname, ip);
    Ok(())
}

fn get_host_by_name(hostname: &str) -> Result<IpAddr, Box<dyn Error>> {
    if let Ok(mut sock_addrs) = (hostname, 0).to_socket_addrs() {
        if let Some(sock_addr) = sock_addrs.next() {
            return Ok(sock_addr.ip());
        }
    }
    Err("No IP address found")?
}