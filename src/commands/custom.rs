use crate::{Data, Error};
use poise::serenity_prelude as serenity;

type Context<'a> = poise::Context<'a, Data, Error>;

// Add new commands in this file