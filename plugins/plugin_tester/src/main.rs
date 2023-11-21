#![feature(let_chains)]
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{
    env::args,
    fs::{canonicalize, metadata},
    mem::size_of,
    path::PathBuf,
    process::Stdio,
    str::from_utf8,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
};

use reddw_source_plugin::{SourcePluginMessage, SourcePluginResponse};
use rmp_serde::{from_slice, to_vec};

#[tokio::main]
async fn main() {
    let plugin = canonicalize(PathBuf::from(args().nth(1).unwrap())).unwrap();
    #[cfg(target_family = "unix")]
    {
        let mode = metadata(&plugin)
            .and_then(|m| Ok(m.permissions().mode()))
            .unwrap();
        let mode = format!("{:o}", mode);
        let mode = mode
            .split_at(3)
            .1
            .chars()
            .filter_map(|c| u8::try_from(c).ok())
            .collect::<Vec<_>>();
        if mode.iter().all(|byte| byte % 2 == 0) {
            panic!(
                "Plugin {:?} is not marked as executable. Detected mode: {:?}",
                plugin, mode
            )
        }
    }
    let mut child = Command::new(plugin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();

    stdin
        .write(&(to_vec(&SourcePluginMessage::GetName).unwrap()))
        .await
        .unwrap();
    let response: SourcePluginResponse = {
        let mut buf = [0 as u8; size_of::<SourcePluginResponse>()];
        stdout.read(&mut buf).await.unwrap();
        from_slice(&buf).unwrap()
    };
    let err = {
        let mut err = Vec::<u8>::new();
        stderr.read(&mut err).await.unwrap();
        let str = from_utf8(&err).unwrap();
        str.to_owned()
    };
    println!("{:#?}", response);
    eprintln!("{}", err);
    child.kill().await.unwrap();
}
