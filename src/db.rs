use crate::error::MainError;
use crate::parsing_types::{Text, TextData};
use derive_more::{Display, Error, From};
use lazy_static::lazy_static;
use regex;
use serde::{Deserialize, Serialize};
use sled::IVec;
use smart_default::SmartDefault;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use teloxide::prelude::*;
use teloxide::types::MessageKind;
use teloxide::utils::command::BotCommand;
use tokio::prelude::*;

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct ChatId(pub i64);

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub struct UserId(pub i32);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CodeUser {
    pub username: Option<String>,
    pub firstname: String,
    pub telegram_id: UserId,
    pub codewars_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub id: i32,
    pub text: String,
    pub from: UserId,
}

pub struct Persist {
    users: sled::Db,
    messages: sled::Db,
}

impl Persist {
    pub fn new(db: sled::Db, msg_db: sled::Db) -> Self {
        Self {
            users: db,
            messages: msg_db,
        }
    }

    pub fn add_message(&self, chat_id: ChatId, msg: ChatMessage) -> Result<(), MainError> {
        let mut messages = match self
            .messages
            .get(serde_json::to_vec(&chat_id)?.as_slice())
            .unwrap()
        {
            None => Vec::new(),
            Some(vec) => serde_json::from_slice(vec.as_ref())?,
        };
        messages.push(msg.clone());
        self.messages
            .insert(
                serde_json::to_vec(&chat_id)?.as_slice(),
                serde_json::to_vec(&messages)?.as_slice(),
            )
            .unwrap();
        log::info!("message {:?} added to chat {:?}", &msg, &chat_id);
        Ok(())
    }

    pub fn add_user(&self, chat_id: ChatId, user: CodeUser) -> Result<(), MainError> {
        let mut map = match self
            .users
            .get(serde_json::to_vec(&chat_id)?.as_slice())
            .unwrap()
        {
            None => HashMap::new(),
            Some(val) => serde_json::from_slice(val.as_ref())?,
        };
        let user1 = user.clone();
        map.insert(user1.telegram_id, user1);
        self.users
            .insert(
                serde_json::to_vec(&chat_id)?.as_slice(),
                serde_json::to_vec(&map)?.as_slice(),
            )
            .unwrap();
        log::info!("user {:?} added in chat {:?}", &user, &chat_id);
        Ok(())
    }

    pub fn remove_user(&self, chat_id: ChatId, user_to_remove: UserId) -> Result<(), MainError> {
        let mut users: HashMap<UserId, CodeUser> = self
            .users
            .get(serde_json::to_vec(&chat_id)?.as_slice())
            .unwrap()
            .map_or(Ok(HashMap::new()), |v| -> Result<_, serde_json::Error> {
                Ok(serde_json::from_slice(v.as_ref())?)
            })?;
        users.remove(&user_to_remove);
        self.users
            .insert(
                serde_json::to_vec(&chat_id)?.as_slice(),
                serde_json::to_vec(&users)?.as_slice(),
            )
            .unwrap();
        log::info!("user {:?} removed in chat {:?}", &user_to_remove, &chat_id);
        Ok(())
    }

    pub fn clear_users(&self, chat_id: ChatId) -> Result<(), MainError> {
        self.users.insert(
            serde_json::to_vec(&chat_id)?.as_slice(),
            serde_json::to_vec(&HashMap::<UserId, CodeUser>::new())?.as_slice(),
        )?;
        log::info!("users cleared in chat {:?}", &chat_id);
        Ok(())
    }

    pub fn clear_messages(&self, chat_id: ChatId) -> Result<(), MainError> {
        self.messages.insert(
            serde_json::to_vec(&chat_id)?.as_slice(),
            serde_json::to_vec(&Vec::<ChatMessage>::new())?.as_slice(),
        )?;
        log::info!("messages cleared in chat {:?}", &chat_id);
        Ok(())
    }

    pub fn get_users(&self, chat_id: ChatId) -> Result<HashMap<UserId, CodeUser>, MainError> {
        Ok(self
            .users
            .get(serde_json::to_vec(&chat_id)?.as_slice())
            .unwrap()
            .map_or(Ok(HashMap::new()), |v| -> Result<_, serde_json::Error> {
                Ok(serde_json::from_slice(v.as_ref())?)
            })?)
    }

    pub fn get_messages(&self, chat_id: ChatId) -> Result<Vec<ChatMessage>, MainError> {
        Ok(
            match self
                .messages
                .get(serde_json::to_vec(&chat_id)?.as_slice())
                .unwrap()
            {
                Some(vec) => serde_json::from_slice(vec.as_ref())?,
                None => Vec::new(),
            },
        )
    }
}
