use serde::Serialize;
use std::env;
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig, SecureTCPTransport,
    TwitchIRCClient,
};

#[derive(Debug, Serialize)]
struct ChatMessage {
    pub timestamp: i64,
    pub channel: String,
    pub user: String,
    pub content: String,
    pub display_name: String,
}

#[tokio::main]
async fn main() {
    let url = "https://api.op47.de/v1/twitch/insertMessage";
    let look_in = vec!["stegi", "di1araas"];
    let look_for = vec!["stegi", "di1araas"];

    let api_key = env::var("API_KEY").expect("API_KEY must be set");
    let bot_name = env::var("BOT_NAME");
    let bot_oauth = env::var("BOT_OAUTH");
    match (bot_name, bot_oauth) {
        (Ok(bot_name), Ok(bot_oauth)) => {
            println!("Bot name: {}", bot_name);
            println!("Bot oauth: {}", bot_oauth);
        }
        _ => {
            println!("The environment variables BOT_NAME and BOT_OAUTH were not set using anonymous user");
        }
    }

    let config = ClientConfig::default();
    let (mut incoming_messages, client) =
        TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    let join_handle = tokio::spawn(async move {
        while let Some(message) = incoming_messages.recv().await {
            handle_message(message, &look_for, &url, &api_key).await;
        }
    });

    for channel in look_in {
        match client.join(channel.to_string()) {
            Ok(_) => println!("Joined channel {}", channel),
            Err(e) => println!("Error joining channel ({}): {}", channel, e),
        }
    }
    join_handle.await.unwrap();
}

async fn handle_message(message: ServerMessage, look_for: &Vec<&str>, url: &str, api_key: &str) {
    match message {
        ServerMessage::Privmsg(msg) => {
            if look_for.contains(&msg.sender.login.as_str()) {
                let chat_message = ChatMessage {
                    timestamp: msg.server_timestamp.timestamp(),
                    channel: "#".to_string() + msg.channel_login.as_str(),
                    user: msg.sender.login.clone(),
                    content: msg.message_text,
                    display_name: msg.sender.name,
                };
                let color = msg.name_color.unwrap();
                println!(
                    "user: {0}; rgb: ({1}, {2}, {3})",
                    &msg.sender.login.as_str(),
                    color.r,
                    color.g,
                    color.b
                );

                match post_message(chat_message, url, api_key).await {
                    Ok(_) => println!("Message posted"),
                    Err(e) => println!("Error posting message: {}", e),
                }
            }
        }
        _ => {}
    };
}

async fn post_message(
    message: ChatMessage,
    url: &str,
    api_key: &str,
) -> Result<(), reqwest::Error> {
    let http_client = reqwest::Client::new();
    let body = serde_json::to_string_pretty(&message).unwrap();
    let _ = http_client
        .post(url)
        .body(body)
        .header("authorization", api_key)
        .header("content-type", "application/json")
        .send()
        .await?;
    Ok(())
}
