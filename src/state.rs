use crate::config::Config;

use std::{collections::HashMap, sync::Arc};

use serenity::{model::id::GuildId, prelude::*};

#[derive(Default)]
pub struct State {
    pub guilds: RwLock<HashMap<GuildId, Arc<RwLock<GuildState>>>>,
}

pub struct GuildState {
    pub config: Config,
    pub markov_chain: markov::Chain<String>,
}

impl Default for GuildState {
    fn default() -> Self {
        Self {
            config: Default::default(),
            markov_chain: markov::Chain::new(),
        }
    }
}

/// Field of `serenity::prelude::Context::data` used to store the state in the
/// context.
impl TypeMapKey for State {
    type Value = State;
}
