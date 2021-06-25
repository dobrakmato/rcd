use clap::{App, Arg};
use std::str::FromStr;
use std::net::{TcpStream, SocketAddr};
use std::io::{Write, Read};
use std::time::Duration;

fn main() {
    let matches = App::new("rc")
        .version("1.0")
        .author("Matej Kormuth <dobrakmato@gmail.com")
        .arg(Arg::with_name("target")
            .short("t")
            .long("target")
            .value_name("TARGET")
            .help("Target IP address")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("timeout")
            .short("w")
            .long("timeout")
            .value_name("TIMEOUT")
            .help("Connection timeout in seconds")
            .takes_value(true))
        .arg(Arg::with_name("cmd")
            .short("c")
            .long("cmd")
            .value_name("CMD")
            .help("Command to execute")
            .required(true)
            .takes_value(true))
        .arg(Arg::with_name("password")
            .short("p")
            .long("password")
            .value_name("PASSWORD")
            .help("Password used for access")
            .required(true)
            .takes_value(true))
        .get_matches();

    let timeout = matches.value_of("timeout").unwrap_or("1.0").parse::<f32>()
        .expect("cannot parse timeout as seconds value");
    let pwd = matches.value_of("password").unwrap();
    let target = matches.value_of("target").unwrap();
    let cmd = u8::from_str(matches.value_of("cmd").unwrap())
        .expect("invalid command. only valid u8 numbers are allowed.");

    let target = format!("{}:7305", target);

    if pwd.len() != 32 {
        panic!("password length must be 32 bytes")
    }

    let addr = SocketAddr::from_str(&target).expect("cannot parse target");
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs_f32(timeout))
        .expect("cannot connect to target");
    stream.write_all(pwd.as_bytes()).expect("cannot write password");
    stream.write_all(&mut [cmd]).expect("cannot write cmd");
    let mut result = Vec::new();
    stream.read_to_end(&mut result).expect("cannot read result");

    println!("{}", String::from_utf8_lossy(result.as_slice()));
}
