use std::{
    env::{args_os, current_exe},
    ffi::OsString,
    fs::write,
    io::{self, Write},
    path::PathBuf,
    process::{exit, Command, Stdio},
    thread,
    time::Duration,
};

fn descendant(arguments: Vec<OsString>) {
    let path = arguments.first().map(PathBuf::from).unwrap_or_default();
    let delay = arguments.get(1).cloned();
    let hold_pipes = arguments.get(2).is_some_and(|value| value == "hold-pipes");
    let mut command = Command::new(current_exe().expect("current helper executable"));
    command.arg("sentinel").arg(path).arg(delay.unwrap_or_else(|| OsString::from("500")));
    if hold_pipes {
        command.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    } else {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }
    command.spawn().expect("spawn sentinel descendant");
    println!("ready");
    io::stdout().flush().expect("flush readiness");
    if !arguments.get(3).is_some_and(|value| value == "exit-leader") {
        thread::sleep(Duration::from_secs(30));
    }
}
fn dual(count: Option<OsString>) {
    let count = usize::try_from(parse_u64(count)).unwrap_or(usize::MAX);
    let stdout = thread::spawn(move || {
        let mut output = io::stdout().lock();
        output.write_all(&vec![b'o'; count]).expect("write stdout");
        output.write_all(&[0xff]).expect("write non-UTF-8 stdout");
    });
    let stderr = thread::spawn(move || {
        let mut output = io::stderr().lock();
        output.write_all(&vec![b'e'; count]).expect("write stderr");
        output.write_all(&[0xfe]).expect("write non-UTF-8 stderr");
    });
    stdout.join().expect("join stdout writer");
    stderr.join().expect("join stderr writer");
}
fn main() {
    let mut arguments = args_os().skip(1);
    let mode = arguments.next().unwrap_or_default();
    match mode.to_string_lossy().as_ref() {
        | "args" => print_arguments(arguments.collect()),
        | "descendant" => descendant(arguments.collect()),
        | "dual" => dual(arguments.next()),
        | "exit" => exit(parse_u64(arguments.next()) as i32),
        | "sentinel" => sentinel(arguments.collect()),
        | "sleep" => thread::sleep(Duration::from_millis(parse_u64(arguments.next()))),
        | "touch" => touch(arguments.next()),
        | _ => exit(64),
    }
}
fn parse_u64(value: Option<OsString>) -> u64 {
    value
        .and_then(|value| value.into_string().ok())
        .and_then(|value| value.parse().ok())
        .unwrap_or_default()
}
fn print_arguments(arguments: Vec<OsString>) {
    arguments.into_iter().for_each(|argument| {
        let value = argument.to_string_lossy();
        println!("{}:{value}", value.len());
    });
}
fn sentinel(arguments: Vec<OsString>) {
    let path = arguments.first().map(PathBuf::from).unwrap_or_default();
    let delay = parse_u64(arguments.get(1).cloned());
    thread::sleep(Duration::from_millis(delay));
    write(path, b"survived").expect("write sentinel");
}
fn touch(path: Option<OsString>) {
    write(PathBuf::from(path.unwrap_or_default()), b"spawned").expect("write touch marker");
}
