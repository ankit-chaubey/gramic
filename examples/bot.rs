//! cargo run --example bot -- [serve | set | delete | info]
//! needs TOKEN and WEBHOOK_URL in env or .env
//!
//! Cargo.toml:
//!   gramic = "0.1"
//!   tokio  = { version = "1", features = ["full"] }

use gramic::{Bot, Update};

#[tokio::main]
async fn main() {
    load_dotenv();

    let token  = var("TOKEN");
    let url    = std::env::var("WEBHOOK_URL").unwrap_or_default();
    let port   = std::env::var("PORT").unwrap_or_else(|_| "8080".into()).parse().unwrap();
    let secret = std::env::var("SECRET").unwrap_or_default();
    let path   = std::env::var("WEBHOOK_PATH").unwrap_or_else(|_| "/webhook".into());

    let mut bot = Bot::new(&token, &url).port(port).path(&path);
    if !secret.is_empty() { bot = bot.secret(&secret); }

    match std::env::args().nth(1).as_deref().unwrap_or("serve") {
        "serve"  => bot.serve(on_update).await.unwrap(),
        "set"    => bot.set().await.unwrap(),
        "delete" => bot.delete().await.unwrap(),
        "info"   => {
            let i = bot.info().await.unwrap();
            println!("url     : {}", i.url);
            println!("pending : {}", i.pending_update_count);
            println!("error   : {}", i.last_error_message.as_deref().unwrap_or("none"));
        }
        other => {
            eprintln!("unknown: {other}  (serve | set | delete | info)");
            std::process::exit(1);
        }
    }
}

async fn on_update(u: Update) {
    if let Some(msg) = u.message {
        let text = msg.text.as_deref().unwrap_or("");
        let from = msg.from.as_ref().map(|u| u.first_name.as_str()).unwrap_or("?");
        println!("[msg] chat={}  from={from}  text={text:?}", msg.chat.id);
    } else if let Some(cbq) = u.callback_query {
        println!("[cbq] id={}  data={:?}", cbq.id, cbq.data);
    } else if let Some(msg) = u.edited_message {
        println!("[edit] chat={}  text={:?}", msg.chat.id, msg.text);
    } else if let Some(post) = u.channel_post {
        println!("[channel] chat={}  text={:?}", post.chat.id, post.text);
    }
}

fn var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{key} not set"))
}

fn load_dotenv() {
    if let Ok(s) = std::fs::read_to_string(".env") {
        for line in s.lines() {
            if let Some((k, v)) = line.split_once('=') {
                let k = k.trim();
                let v = v.trim().trim_matches('"');
                if !k.starts_with('#') { std::env::set_var(k, v); }
            }
        }
    }
}
