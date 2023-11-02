use discord_sdk as ds;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::time::SystemTime;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

pub mod discord;

const DISCORD_APP_ID: i64 = 1169653566680608899;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Event {
    FileChange(FileChangeEvent),
}
#[derive(Debug, Deserialize)]
struct FileChangeEvent {
    pub file_name: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
struct Response {
    msg: String,
}

impl Response {
    fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_owned(),
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str: String = self.into();
        write!(f, "{}", str)
    }
}

impl From<&Response> for String {
    fn from(r: &Response) -> Self {
        serde_json::to_string(r).unwrap()
    }
}
impl From<Response> for String {
    fn from(r: Response) -> Self {
        serde_json::to_string(&r).unwrap()
    }
}

impl FileChangeEvent {
    fn get_lang_name(&self) -> Option<&str> {
        let ext = self.file_name.split('.').last()?;
        match ext {
            "rs" => Some("rust"),
            "json" => Some("json"),
            "js" => Some("javascript"),
            "ts" => Some("typescript"),
            "py" => Some("python"),
            "lua" => Some("lua"),
            _ => None,
        }
    }
}
fn get_activity_builder(
    details: &str,
    small_img: (&str, &str),
    sys_time: SystemTime,
) -> ds::activity::ActivityBuilder {
    ds::activity::ActivityBuilder::default()
        .details(details.to_owned())
        .assets(
            ds::activity::Assets::default()
                .small(small_img.0.to_owned(), Some(small_img.1.to_owned()))
                .large("neovim-logo".to_owned(), Some("neovim")),
        )
        .start_timestamp(sys_time)
}
#[tokio::main]
async fn main() {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    let stdout = io::stdout();
    let mut writer = io::BufWriter::new(stdout);

    let mut start_time: Option<SystemTime> = None;
    let client = discord::make_client(discord_sdk::Subscriptions::ACTIVITY).await;

    while let Some(line) = reader.next_line().await.unwrap() {
        let request = line;
        let event: Event = match serde_json::from_str(&request) {
            Ok(e) => e,
            Err(e) => {
                let res = Response::new(&format!("error: {}", e));
                let _ = writer.write_all(format!("{}\n", res).as_bytes()).await;
                continue;
            }
        };

        let res = match event {
            Event::FileChange(fce) => {
                if start_time.is_none() {
                    start_time.replace(SystemTime::now());
                }
                let lang_name = fce.get_lang_name().unwrap_or("None");
                let title = format!("rawdogging {}", fce.file_name);
                let rp = get_activity_builder(
                    &title,
                    (lang_name, lang_name),
                    start_time.unwrap_or(SystemTime::now()),
                );
                let _ = client.discord.update_activity(rp).await;
                Response::new("ok")
            }
        };
        let _ = writer.write_all(format!("{}\n", res).as_bytes()).await;
    }
    client.discord.disconnect().await;
}
