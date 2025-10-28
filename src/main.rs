use std::fmt::format;
use std::io::{Read, Write};
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
    serials: Vec<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    // // Connect to the server
    // let hostname = format!("{}:9100", args.hostname[0]);
    // let mut stream = TcpStream::connect(hostname)?;
    // println!("Connected to the server!");

    // // Send data (as bytes)
    // let request = b"^XA^HH^XZ";
    // stream.write_all(request)?;
    // println!("Sent: {}", String::from_utf8_lossy(request));

    // // Wait for a response
    // let mut buffer = [0; 2048]; // Adjust buffer size as needed
    // let bytes_read = stream.read(&mut buffer)?;
    // println!("Received: {}", String::from_utf8_lossy(&buffer[..bytes_read]));

    for serial in args.serials {
        let serial = serial.trim_start_matches("csc");
        match get_host_by_name(&serial) {
            Ok(ip) => {
                println!("{}: {}", serial, ip);
                update_hostname(&ip, &serial)?;
                let hostname = format!("csc{}", serial);
                print_hostname_ip(&ip, &hostname)?;
                restart_printer(&ip)?;
            },
            Err(e) => {
                eprintln!("Error resolving {}: {}", serial, e);
                let hostname = format!("csc{}", serial);
                match get_host_by_name(&hostname) {
                    Ok(ip) => {
                        println!("{}: {}", hostname, ip);
                        print_hostname_ip(&ip, &hostname)?;
                    },
                    Err(e) => {
                        eprintln!("Error resolving {}: {}", hostname, e)
                    }
                }
            }
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

    let data = format!("^XA^FO60,40^BAN,60,Y,Y,N,N^FD>:{}^FS^FO60,130^BAN,60,Y,Y,N,N^FD>:{}^FS^XZ", hostname, ip);
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