use poise::serenity_prelude::{self as serenity, GatewayIntents};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use std::time::Instant;
use std::fs;
use toml;
use std::sync::atomic::{AtomicBool, Ordering};

mod commands;

fn get_command_from_string(command_name: &str) -> Option<poise::Command<Data, Error>> {
    match command_name {
        "ping" => Some(ping()),
        "help" => Some(commands::general::help()),
        "about" => Some(commands::general::about()),
        "avatar" => Some(commands::general::avatar()),
        "server_info" => Some(commands::general::server_info()),
        // Add new commands here
        _ => None,
    }
}

// Example command:
/// Returns pong and the latency of the message
#[poise::command(slash_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {

    let start = Instant::now();

    ctx.say("Pong!").await?;

    let latency = start.elapsed();

    ctx.say(format!("Latency: {:.2?}", latency)).await?;

    if DEBUG_MODE.load(Ordering::SeqCst) {
        println!("ping command executed by {}, latency {:.2?}.", ctx.author(), latency);
    }    
    
    Ok(())
}

#[derive(Deserialize)]
struct BotConfig {
    name: String,
    developer: String,
    description: String,
    token: String,
    commands: Vec<String>,
}

#[derive(Deserialize)]
struct Config {
    bot: BotConfig,
}

async fn read_config(file_path: &str) -> Result<Config, toml::de::Error> {
    let content = fs::read_to_string(file_path).expect("Failed to read file");

    toml::de::from_str(&content)
}

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Parser)]
#[clap(name = "librebot")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Start,
}

static DEBUG_MODE: AtomicBool = AtomicBool::new(false);

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = read_config("bot.toml");

    match cli.command {
        Command::Start => {
            match config.await {
                Ok(cfg) => {
                   println!("Succesfully read config!\nStarting bot...");
                   let name = cfg.bot.name;
                   let token = cfg.bot.token;
                   let commands = cfg.bot.commands;

                    tokio::join!(
                        start_bot(token, commands),
                        handle_command_line_input(name)
                    );
                },
                Err(e) => eprintln!("Error reading config: {}", e),
            }   
        }
    }
}

async fn start_bot(token: String, command_strings: Vec<String>) {
    let token = token;
    let intents = GatewayIntents::GUILD_MEMBERS | GatewayIntents::GUILDS;

    let commands: Vec<poise::Command<Data, Error>> = command_strings
        .iter()
        .filter_map(|cmd| get_command_from_string(cmd))
        .collect();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: commands,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

async fn handle_command_line_input(bot_name: String) {
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    println!("Command-line input handler started. Type commands below:");

    while let Ok(Some(line)) = lines.next_line().await {
        match line.as_str() {
            "stop" => {
                println!("Stopping the bot...");
                std::process::exit(0);
            }
            "status" => {
                println!("The bot '{}' is running.", bot_name);
            }
            "debug on" => {
                DEBUG_MODE.store(true, Ordering::SeqCst);
                println!("Debug mode enabled.");
            }
            "debug off" => {
                DEBUG_MODE.store(false, Ordering::SeqCst);
                println!("Debug mode disabled.");
            }
            _ => {
                println!("Unknown command: {}", line);
            }
        }
    }
}