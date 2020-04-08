use crate::db::{ChatId, ChatMessage, CodeUser, Persist, UserId};
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

lazy_static! {
    static ref IS_SOLUTION_REGEX: regex::Regex =
        regex::Regex::new(r"^\d\D*https://pastebin.com/").unwrap();
    static ref KATA_KYU: regex::Regex = regex::Regex::new(r"^\d(?:\s*kyu|\s)").unwrap();
    static ref LINK: regex::Regex = regex::Regex::new(r"https://pastebin\.com/(.|\s)*").unwrap();
}

pub fn is_codewars_solution(msg: &str) -> bool {
    IS_SOLUTION_REGEX.is_match(msg)
}

pub fn kata_name(msg: &str) -> String {
    if !is_codewars_solution(msg) {
        panic!("Text {} is not a codewars solution", msg);
    }
    let name_link = KATA_KYU.replace(msg, "");
    let name = LINK.replace(name_link.as_ref(), "");
    name.trim().to_owned()
}