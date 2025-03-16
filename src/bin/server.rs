use evalexpr::eval_int;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, StdinLock, Stdout, Write};

use stdio_rpc::{
    proto_postcard::{C2SMsg, S2CMsg},
    receive_msg, send_msg,
};

fn main() {
    let _ = serve();
}

fn serve() -> io::Result<()> {
    let mut log_file = OpenOptions::new()
        .create(true) // Create the file if it doesn't exist
        .append(true) // Open in append mode
        .open("server.log")?;

    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout();
    loop {
        match receive_msg::<C2SMsg, _>(&mut stdin)? {
            C2SMsg::Request(expression) => {
                writeln!(log_file, "server received request: {expression}")?;
                handle_request(expression, &mut stdin, &mut stdout, &mut log_file)?
            }
            _ => send_msg(&S2CMsg::BadSeq, &mut stdout)?,
        };
    }
}

fn handle_request(
    expression: String,
    stdin: &mut StdinLock<'_>,
    stdout: &mut Stdout,
    log: &mut File,
) -> io::Result<()> {
    writeln!(log, "server handle request")?;

    let mut kv = expression
        .chars()
        .filter(|&ch| !(ch.is_ascii_digit() || ch.is_whitespace() || "+-*/".contains(ch)))
        .zip(std::iter::repeat(0i64))
        .collect::<HashMap<_, _>>();

    for (k, v) in kv.iter_mut() {
        send_msg(&S2CMsg::Query(*k), stdout)?;
        match receive_msg::<C2SMsg, _>(stdin)? {
            C2SMsg::Reply(value) => {
                *v = value;
            }
            _ => return send_msg(&S2CMsg::BadSeq, stdout),
        }
    }

    let mut replaced = expression;
    for (old, new) in kv.iter() {
        replaced = replaced.replace(*old, &new.to_string());
    }
    let ret = eval_int(&replaced).unwrap_or_default();

    send_msg(&S2CMsg::Response(ret), stdout)
}
