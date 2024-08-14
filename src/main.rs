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
struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // We are going to move the guild ID to the Secrets.toml file later.
        let guild_id = GuildId::new(1219770752044367983);

        // We are creating a vector with commands
        // and registering them on the server with the guild ID we have set.
        let commands = vec![CreateCommand::new("hello").description("Say hello")];
        let commands = guild_id.set_commands(&ctx.http, commands).await.unwrap();

        info!("Registered commands: {:#?}", commands);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {

        if let Interaction::Command(command) = interaction {
            let respond_to = &command.user.mention();
            let responce_content = match command.data.name.as_str() {
                "hello" => format!("Hello, {}!", respond_to),
                command => unreachable!("Unknown command: {}", command),
            };

            let data = CreateInteractionResponseMessage::new().content(responce_content);
            let builder = CreateInteractionResponse::Message(data);

            if let Err(why) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command: {why}");
            }
        }
    }

    // LISTENING FOR MESSAGES
    async fn message(&self, _ctx: Context, msg: Message) {
        // Get the message content
        let message_content = msg.content.clone();

        // The file path where messages will be stored
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
            .truncate(true)  // Overwrite the file
            .open(file_path)
            .await
            .expect("Failed to open file");

        if let Err(e) = file.write_all(json_data.as_bytes()).await {
            eprintln!("Failed to write to file: {}", e);
        }
    }
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

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())

    
}
