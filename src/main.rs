
use anyhow::Context as _;
use rand::seq::SliceRandom;

use serenity::all::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction, Message};
use serenity::{all::GuildId, async_trait};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::info;

use tokio::time::{self, Duration};
use serenity::model::id::ChannelId;

mod json_utils;
use json_utils::{read_messages_from_file, write_messages_to_file};

// Define the Bot struct
struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // Register commands with the guild
        let guild_id = GuildId::new(1219770752044367983); // Devhub ID for MRHUB HIMSELF
        let commands = vec![CreateCommand::new("hello").description("Say hello")];
        let commands = guild_id.set_commands(&ctx.http, commands).await.unwrap();
        info!("Registered commands: {:#?}", commands);

        // Start the periodic task
        let channel_id = ChannelId::from(1273447883735040051); // Currently in his own channel
        let interval = time::interval(Duration::from_secs(60)); // Sends message into the channel every 60 seconds!! ( he is so smart :> )

        tokio::spawn(async move {
            let mut interval = interval;
            loop {
                interval.tick().await;

                // Generate a sentence and send it
                let messages = read_messages_from_file("message_log.json").await;
                if !messages.is_empty() {
                    let sentence = generate_sentence(&messages);
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
        // Skip messages from bots ( we dont want him to train his own sentences >:( )
        if msg.author.bot {
            return;
        }
        //ty stack overflow
        // Gettin da message conttent
        let message_content = msg.content.clone();
        let file_path = "message_log.json";

        // Load the existing file content
        let mut existing_content = read_messages_from_file(file_path).await;

        // Append the new message content
        if !existing_content.is_empty() {
            existing_content.push(' '); // Add spaces between mesages
        }
        existing_content.push_str(&message_content); //push da message 

        // Write the updated content back to the file
        write_messages_to_file(file_path, &existing_content).await;
    }
}

// function to gernerate the sentences ( CURRENTLY JUST RANDOMLY PICKING WORDS / WOULD MOVE TO MARKOV MAYBE IF MY BRAIN CAN UNDERSTAND IT )
fn generate_sentence(messages: &str) -> String {
    let mut rng = rand::thread_rng();
    let words: Vec<&str> = messages.split_whitespace().collect();
    
    // Create a new sentence by randomly picking words
    
    let sentence: Vec<&str> = (1..20) // range of how many words we want it too choose from ( also ty stackoverflow :D)
        .filter_map(|_| words.choose(&mut rng))
        .copied()
        .collect();
    
    sentence.join(" ")
}
//default shuttle shit
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
