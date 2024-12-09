use crate::{read_config, Data, Error, DEBUG_MODE};
use std::sync::atomic::Ordering;
use poise::serenity_prelude::{self as serenity, CreateEmbed, Mentionable};

type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {

    if DEBUG_MODE.load(Ordering::SeqCst) {
        println!("help command executed by {}.", ctx.author());
    }  

    let embed = CreateEmbed::default()
        .title("Commands")
        .description("The following commands are available:
        **General Commands**
        ```/help``` This command, returns a list of all commands
        ```/ping``` Returns 'Pong!' and the latency of the message indicating that the server is online
        ```/about``` Returns general information about the bot
        ```/avatar``` Returns the user's avatar
        ```/server_info``` Returns various pieces of information about the current guild
        ")
        .color(0x00FF00)
        .timestamp(chrono::Utc::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    
    Ok(())
}

#[poise::command(slash_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {

    if DEBUG_MODE.load(Ordering::SeqCst) {
        println!("about command executed by {}.", ctx.author());
    }  

    let config = read_config("bot.toml");

    match config.await {
        Ok(cfg) => {
            let embed = CreateEmbed::default()
                .title("About")
                .description(format!("Name: {}
                Developer: {}
                Description: {}
                
                This bot is a self-hosted librebot instance, an open-source bot that's easy to self-host and modify!
                ", cfg.bot.name, cfg.bot.developer, cfg.bot.description))
                .color(0x00FF00)
                .timestamp(chrono::Utc::now());

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {

            if DEBUG_MODE.load(Ordering::SeqCst) {
                println!("Error reading config file in about command executed by {}.
            {}", ctx.author(), e);
            }  

            let embed = CreateEmbed::default()
                .title("Error")
                .description(format!("An error occured in reading the bot.toml file: {}", e))
                .color(0xFF0000)
                .timestamp(chrono::Utc::now());

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }
    
    Ok(())
}

#[poise::command(slash_command)]
pub async fn avatar(ctx: Context<'_>) -> Result<(), Error> {

    if DEBUG_MODE.load(Ordering::SeqCst) {
        println!("avatar command executed by {}.", ctx.author());
    } 

    let avatar_url = ctx.author().avatar_url().unwrap_or(ctx.author().default_avatar_url());

    let embed = CreateEmbed::default()
        .title(format!("{}'s Avatar", ctx.author().mention()))
        .image(avatar_url)
        .color(0x00FF00)
        .timestamp(chrono::Utc::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    
    Ok(())
}

#[poise::command(slash_command)]
pub async fn server_info(ctx: Context<'_>) -> Result<(), Error> {
    if DEBUG_MODE.load(Ordering::SeqCst) {
        println!("server_info command executed by {}.", ctx.author());
    }
    
    if let Some(guild_id) = ctx.guild_id() {
        if let Ok(guild) = ctx.serenity_context().http.get_guild(guild_id).await {
            // Handle guild owner mention more safely
            let guild_owner = match guild
                .owner_id
                .to_user(ctx.serenity_context())
                .await {
                    Ok(user) => user.mention().to_string(),
                    Err(_) => "Unknown".to_string(),
            };

            let boost_tier = match guild.premium_tier {
                serenity::PremiumTier::Tier1 => "1",
                serenity::PremiumTier::Tier2 => "2",
                serenity::PremiumTier::Tier3 => "3",
                _ => "0",
            };

            let verification_level = match guild.verification_level {
                serenity::VerificationLevel::None => "None",
                serenity::VerificationLevel::Low => "Low",
                serenity::VerificationLevel::Medium => "Medium",
                serenity::VerificationLevel::High => "High",
                serenity::VerificationLevel::Higher => "Higher",
                _ => "Unknown",
            };

            let members = guild.members(&ctx.serenity_context().http, None, None).await?;
            let member_count = members.len();

            let embed = CreateEmbed::default()
                .title(&guild.name)
                .description(format!(
                    "Name: {}
                    ID: {}
                    Owner: {}
                    Member Count: {}
                    Channel Count: {}
                    Role Count: {}
                    Boosts: {} (level {})
                    Created at: {}
                    Verification Level: {}
                    Emoji count: {}",
                    guild.name,
                    guild.id,
                    guild_owner,
                    member_count,
                    guild.channels(ctx.serenity_context()).await?.len(),
                    guild.roles.len(),
                    guild.premium_subscription_count.unwrap_or(0),
                    boost_tier,
                    guild.id.created_at(),
                    verification_level,
                    guild.emojis.len(),
                ))
                .image(guild.icon_url().unwrap_or_default())
                .color(0x00FF00)
                .timestamp(chrono::Utc::now());

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        } else {
            ctx.say("Could not fetch the guild data.").await?;
        }
    } else {
        ctx.say("This command can only be used in a guild!").await?;
    }
    Ok(())
}