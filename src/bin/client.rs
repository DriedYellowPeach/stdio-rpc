use colored::Colorize;
use lazy_static::lazy_static;

use std::collections::HashMap;
use std::io::BufReader;
use std::process::{ChildStdin, ChildStdout, Command, Stdio};

use stdio_rpc::proto_postcard::{C2SMsg, S2CMsg};
use stdio_rpc::{receive_msg, send_msg};

lazy_static! {
    static ref TOKEN: HashMap<char, i64> = {
        let map = [
            ('a', 1),
            ('b', 2),
            ('c', 3),
            ('▲', 1),
            ('▼', -1),
            ('▶', 100),
            ('◀', 200),
        ];
        HashMap::from(map)
    };
}

fn main() {
    let mut child = Command::new("./target/debug/server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    let mut child_stdin = child.stdin.take().expect("Failed to open stdin");
    let mut child_stdout = child.stdout.take().expect("Failed to open stdout");

    println!("{}", "Symbols Pool:".bold().blue());
    TOKEN.iter().enumerate().for_each(|(idx, (c, v))| {
        print!("{: <3}:{: >4}    ", c.to_string().cyan(), v);
        if idx % 3 == 2 {
            println!();
        }
    });
    println!("\n");

    loop {
        if let Err(e) = run(&mut child_stdin, &mut child_stdout) {
            eprintln!("Error: {}", e);
            break;
        }
    }

    let _ = child.wait();
}

fn run(child_in: &mut ChildStdin, child_out: &mut ChildStdout) -> std::io::Result<()> {
    println!("Enter an expression: ");
    let mut expression = String::new();
    std::io::stdin().read_line(&mut expression)?;
    let expression = expression.trim();
    println!("\n{: ^8}{: ^21}{: ^8}", "client", "", "server");

    send_msg(&C2SMsg::Request(expression.to_string()), child_in)?;
    println!("{}", &C2SMsg::Request(expression.to_string()));

    let mut reader = BufReader::new(child_out);

    loop {
        match receive_msg::<S2CMsg, _>(&mut reader)? {
            S2CMsg::Response(ret) => {
                println!("{}", &S2CMsg::Response(ret));
                break;
            }
            S2CMsg::Query(c) => {
                println!("{}", &S2CMsg::Query(c));
                let value = TOKEN.get(&c).copied().unwrap_or_default();
                send_msg(&C2SMsg::Reply(value), child_in)?;
                println!("{}", &C2SMsg::Reply(value));
            }
            S2CMsg::Log(msg) => println!("Log: {}", msg),
            S2CMsg::BadSeq => eprintln!("Bad sequence"),
        }
    }

    println!();

    Ok(())
}
