#![feature(let_chains)]
use reddw_source_plugin::{SourcePluginMessage, SourcePluginResponse};
use rmp_serde::{from_slice, to_vec};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::{
    env::args,
    error::Error,
    fs::{canonicalize, metadata},
    path::PathBuf,
    process::Stdio,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::{ChildStdin, ChildStdout, Command},
};

async fn message(
    stdin: &mut ChildStdin,
    stdout: &mut ChildStdout,
    message: SourcePluginMessage,
) -> Result<SourcePluginResponse, Box<dyn Error>> {
    stdin.write(&(to_vec(&message)?)).await?;
    const BUF_SIZE: usize = 4000;
    let mut vec = Vec::<u8>::new();
    let mut buf = [0 as u8; BUF_SIZE];
    while let Ok(read) = stdout.read(&mut buf).await {
        vec.write(buf.take(BUF_SIZE as u64).get_ref()).await?;
        if read < BUF_SIZE {
            break;
        }
    }
    Ok(from_slice(&vec)?)
}

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
        // .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();
    // let mut stderr = child.stderr.take().unwrap();

    let response = message(&mut stdin, &mut stdout, SourcePluginMessage::GetName).await;
    println!("{:#?}", response);
    message(
        &mut stdin,
        &mut stdout,
        SourcePluginMessage::RegisterInstance("".to_string(), Vec::new()),
    )
    .await
    .unwrap();
    let response = message(
        &mut stdin,
        &mut stdout,
        SourcePluginMessage::GetWallpapers("".to_string()),
    )
    .await;
    println!("{:#?}", response);
    // let err = {
    //     let mut err = Vec::<u8>::new();
    //     stderr.read(&mut err).await.unwrap();
    //     let str = from_utf8(&err).unwrap();
    //     str.to_owned()
    // };
    // eprintln!("Plugin stderr: {:#?}", err);
    child.kill().await.unwrap();
}
