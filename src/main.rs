use dotenv::dotenv;
use reqwest::Client;
use serde_json::{self, json};
use std::env;
use std::process::Command;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let gemini_key = env::var("GEMINI_API_KEY").expect("GEMINI_API_KEY .env missing");

    let output = Command::new("tmux")
        .arg("capture-pane")
        .arg("-p")
        .arg("-S")
        .arg("-100")
        .output()
        .expect("Failed to execute tmux capture");
    let content = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = content.lines().collect();

    let (index, prompt) = lines
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, line)| line.trim_start().contains("$"))
        .nth(1) // first result is calling this script
        .expect("Failed to find prompt");

    let (info, cmd) = prompt
        .find("$")
        .map(|pos| {
            let info = prompt[..pos].split(":").next().unwrap().trim();
            let cmd = prompt[pos + 2..].trim();
            (info, cmd)
        })
        .expect("Failed to trim user info/cmd");

    let relevant_lines: Vec<&str> = lines[index..]
        .iter()
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with(info))
        .cloned()
        .collect();

    let req_body = json!({
        "contents": [
            {
                "parts": [
                    {
                        "text": cmd.to_string() + &relevant_lines.join("\n")
                    }
                ]
            }
        ]
    });

    let client = Client::new();
    let gemini_url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}", gemini_key);
    let res = client
        .post(&gemini_url)
        .header("Content-Type", "application/json")
        .body(req_body.to_string())
        .send()
        .await
        .unwrap();

    if res.status().is_success() {
        let body = res.text().await.unwrap();
        match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(json) => {
                let pretty_json = serde_json::to_string_pretty(&json).unwrap();
                println!("{}", pretty_json);
            }
            Err(e) => {
                println!("Response is not valid JSON: {}", e);
                println!("{}", body);
            }
        }
    } else {
        eprintln!("Req failed: {}", res.status());
    }
}
