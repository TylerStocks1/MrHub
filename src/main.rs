use rand::seq::SliceRandom; // Correct import for SliceRandom
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use anyhow::Context as _;
use serenity::all::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction, Message};
use serenity::{all::GuildId, async_trait};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::info;
use serde_json::Value;
use std::path::Path;
use tokio::time::{self, Duration};
use serenity::model::id::ChannelId;

mod json_utils;
use json_utils::read_messages_from_file;

// Define the Bot struct
struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // Register commands with the guild
        let guild_id = GuildId::new(1219770752044367983); //server ID
        let commands = vec![CreateCommand::new("hello").description("Say hello")];
        let commands = guild_id.set_commands(&ctx.http, commands).await.unwrap();
        info!("Registered commands: {:#?}", commands);

        // Start the periodic task
        let channel_id = ChannelId::from(1227766426761429123); // Main channel ID
        let interval = time::interval(Duration::from_secs(60)); // Adjust the duration as needed

        tokio::spawn(async move {
            let mut interval = interval;
            loop {
                interval.tick().await;

                // Generate a sentence and send it
                let messages = read_messages_from_file("message_log.json");
                if !messages.is_empty() {
                    let sentence = generate_sentence(messages);
                    if let Err(why) = channel_id.say(&ctx.http, sentence).await {
                        eprintln!("Error sending message: {:?}", why);
                    }
                }
            }
        });
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let respond_to = command.user.mention();
            let response_content = match command.data.name.as_str() {
                "hello" => format!("Hello, {}!", respond_to),
                _ => unreachable!("Unknown command"),
            };

            let data = CreateInteractionResponseMessage::new().content(response_content);
            let builder = CreateInteractionResponse::Message(data);

            if let Err(why) = command.create_response(&ctx.http, builder).await {
                eprintln!("Cannot respond to slash command: {:?}", why);
            }
        }
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        // Get the message content
        let message_content = msg.content.clone();
        let file_path = "message_log.json";

        // Load the existing JSON array or create a new one
        let mut messages = if Path::new(file_path).exists() {
            let mut file = File::open(file_path).await.expect("Failed to open file");
            let mut contents = String::new();
            file.read_to_string(&mut contents).await.expect("Failed to read file");

            // If file is empty or invalid, start with an empty array
            if contents.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&contents).unwrap_or_else(|_| Vec::new())
            }
        } else {
            Vec::new()
        };

        // Append the new message content to the array
        messages.push(Value::String(message_content));

        // Serialize the updated array back to JSON
        let json_data = serde_json::to_string_pretty(&messages).expect("Failed to serialize messages");

        // Write the updated JSON array back to the file
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true) // Overwrite the file
            .open(file_path)
            .await
            .expect("Failed to open file");

        if let Err(e) = file.write_all(json_data.as_bytes()).await {
            eprintln!("Failed to write to file: {}", e);
        }
    }
}

// Define the function to generate sentences
fn generate_sentence(messages: Vec<String>) -> String {
    let mut rng = rand::thread_rng();
    let words: Vec<&str> = messages.iter().flat_map(|msg| msg.split_whitespace()).collect();
    
    // Create a new sentence by randomly picking words
    let sentence: Vec<&str> = (0..10) // Choose how many words you want in the new sentence
        .filter_map(|_| words.choose(&mut rng))
        .copied()
        .collect();
    
    sentence.join(" ")
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // Create and start the client
    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Error creating client");

    Ok(client.into())
}
