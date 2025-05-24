use serenity::{
    model::prelude::GuildId as SerenityGuildId,
    prelude::{RwLock, TypeMap, TypeMapKey},
};
use std::{collections::HashMap, sync::Arc};

use crate::heroku_mia::types::Message;

pub(crate) struct ConversationHistory;

impl TypeMapKey for ConversationHistory {
    type Value = Arc<RwLock<HashMap<u64, Vec<Message>>>>;
}

impl ConversationHistory {
    pub async fn get(data: &Arc<RwLock<TypeMap>>) -> Arc<RwLock<HashMap<u64, Vec<Message>>>> {
        let data = data.read().await;
        data.get::<Self>()
            .expect("Expected ConversationHistory in TypeMap")
            .clone()
    }
}

pub(crate) struct GuildId;

impl TypeMapKey for GuildId {
    type Value = SerenityGuildId;
}

impl GuildId {
    pub async fn get(data: &Arc<RwLock<TypeMap>>) -> SerenityGuildId {
        let data = data.read().await;
        *data.get::<Self>().expect("Expected GuildId in TypeMap")
    }
}

pub(crate) struct InferenceModelId;

impl TypeMapKey for InferenceModelId {
    type Value = String;
}

impl InferenceModelId {
    pub async fn get(data: &Arc<RwLock<TypeMap>>) -> String {
        let data = data.read().await;
        data.get::<Self>()
            .expect("Expected InferenceModelId")
            .clone()
    }
}

pub(crate) struct HerokuMiaClient;

impl TypeMapKey for HerokuMiaClient {
    type Value = crate::heroku_mia::Client;
}

impl HerokuMiaClient {
    pub async fn get(data: &Arc<RwLock<TypeMap>>) -> crate::heroku_mia::Client {
        let data = data.read().await;
        data.get::<Self>()
            .expect("Expected HerokuMiaClient")
            .clone()
    }
}

pub(crate) struct AgentTools;

impl TypeMapKey for AgentTools {
    type Value = Vec<crate::heroku_mia::agents::AgentTool>;
}

impl AgentTools {
    pub async fn get(data: &Arc<RwLock<TypeMap>>) -> Vec<crate::heroku_mia::agents::AgentTool> {
        let data = data.read().await;
        data.get::<Self>().expect("Expected AgentTools").clone()
    }
}
