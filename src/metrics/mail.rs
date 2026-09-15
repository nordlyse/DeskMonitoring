use std::path::Path;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::config::Config;
use crate::metrics::apply_mail;
use crate::snapshot::SharedSnapshot;

pub async fn refresh(config: &Config, snapshot: &SharedSnapshot) {
    if config.mail_ready() {
        match fetch_imap(config).await {
            Ok((incoming, outgoing, unread)) => {
                apply_mail(snapshot, incoming, outgoing, unread);
                return;
            }
            Err(_) => {}
        }
    }
    if let Some((incoming, outgoing, unread)) = scan_maildir() {
        apply_mail(snapshot, incoming, outgoing, unread);
    } else {
        apply_mail(snapshot, 0, 0, 0);
    }
}

async fn fetch_imap(config: &Config) -> Result<(u32, u32, u32), String> {
    let host = config.imap_host.trim().to_string();
    let port = config.imap_port;
    let user = config.imap_user.trim().to_string();
    let pass = config.imap_password.clone();
    let inbox = nonempty_mailbox(&config.imap_inbox, "INBOX");
    let sent = nonempty_mailbox(&config.imap_sent, "Sent");

    tokio::time::timeout(
        std::time::Duration::from_secs(20),
        imap_status(&host, port, &user, &pass, &inbox, &sent),
    )
    .await
    .map_err(|_| "IMAP timed out".to_string())?
}

fn nonempty_mailbox(value: &str, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

async fn imap_status(
    host: &str,
    port: u16,
    user: &str,
    pass: &str,
    inbox: &str,
    sent: &str,
) -> Result<(u32, u32, u32), String> {
    let tcp = TcpStream::connect((host, port))
        .await
        .map_err(|e| e.to_string())?;
    let connector = tokio_native_tls::TlsConnector::from(
        native_tls::TlsConnector::new().map_err(|e| e.to_string())?,
    );
    let tls = connector
        .connect(host, tcp)
        .await
        .map_err(|e| e.to_string())?;
    let (reader, mut writer) = tokio::io::split(tls);
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    read_line(&mut reader, &mut line).await?;
    send(
        &mut writer,
        &format!("A1 LOGIN {} {}\r\n", imap_quote(user), imap_quote(pass)),
    )
    .await?;
    expect_tagged(&mut reader, &mut line, "A1").await?;

    send(
        &mut writer,
        &format!("A2 STATUS {} (MESSAGES UNSEEN)\r\n", imap_quote(inbox)),
    )
    .await?;
    let inbox_status = collect_until_tag(&mut reader, &mut line, "A2").await?;
    let (incoming, unread) = parse_status(&inbox_status);

    send(
        &mut writer,
        &format!("A3 STATUS {} (MESSAGES)\r\n", imap_quote(sent)),
    )
    .await?;
    let sent_status = collect_until_tag(&mut reader, &mut line, "A3").await?;
    let (outgoing, _) = parse_status(&sent_status);

    let _ = send(&mut writer, "A4 LOGOUT\r\n").await;
    Ok((incoming, outgoing, unread))
}

fn imap_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

async fn send<W: tokio::io::AsyncWrite + Unpin>(writer: &mut W, cmd: &str) -> Result<(), String> {
    writer
        .write_all(cmd.as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    writer.flush().await.map_err(|e| e.to_string())
}

async fn read_line<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    line: &mut String,
) -> Result<String, String> {
    line.clear();
    let n = reader.read_line(line).await.map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("IMAP closed the connection".to_string());
    }
    Ok(line.trim_end().to_string())
}

async fn expect_tagged<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    line: &mut String,
    tag: &str,
) -> Result<(), String> {
    loop {
        let got = read_line(reader, line).await?;
        if got.starts_with(tag) {
            if got[tag.len()..].trim_start().starts_with("OK") {
                return Ok(());
            }
            return Err(got);
        }
    }
}

async fn collect_until_tag<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    line: &mut String,
    tag: &str,
) -> Result<String, String> {
    let mut blob = String::new();
    loop {
        let got = read_line(reader, line).await?;
        blob.push_str(&got);
        blob.push('\n');
        if got.starts_with(tag) {
            if got[tag.len()..].trim_start().starts_with("OK") {
                return Ok(blob);
            }
            return Err(got);
        }
    }
}

fn parse_status(blob: &str) -> (u32, u32) {
    let mut messages = 0u32;
    let mut unseen = 0u32;
    let upper = blob.to_uppercase();
    if let Some(value) = capture_number(&upper, "MESSAGES") {
        messages = value;
    }
    if let Some(value) = capture_number(&upper, "UNSEEN") {
        unseen = value;
    }
    (messages, unseen)
}

fn capture_number(blob: &str, key: &str) -> Option<u32> {
    let idx = blob.find(key)?;
    let rest = blob[idx + key.len()..].trim_start();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn scan_maildir() -> Option<(u32, u32, u32)> {
    let home = dirs::home_dir()?;
    let roots = [
        home.join("Maildir"),
        home.join("mail"),
        home.join("Mail"),
        home.join(".maildir"),
    ];
    for root in roots {
        if root.is_dir() {
            let incoming = count_dir(&root.join("cur")) + count_dir(&root.join("new"));
            let unread = count_unread(&root.join("cur")) + count_dir(&root.join("new"));
            let outgoing = count_sent(&root);
            return Some((incoming, outgoing, unread));
        }
    }
    None
}

fn count_sent(root: &Path) -> u32 {
    let names = [".Sent", ".Sent Messages", "Sent", ".sent"];
    names
        .iter()
        .map(|name| {
            let dir = root.join(name);
            count_dir(&dir.join("cur")) + count_dir(&dir.join("new"))
        })
        .sum()
}

fn count_dir(path: &Path) -> u32 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| e.path().is_file())
        .count() as u32
}

fn count_unread(cur: &Path) -> u32 {
    let Ok(entries) = std::fs::read_dir(cur) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| {
            let name = e.file_name();
            let lossy = name.to_string_lossy();
            e.path().is_file() && !lossy.contains('S')
        })
        .count() as u32
}
