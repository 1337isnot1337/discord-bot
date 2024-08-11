use crate::{message_top, CONTEXT};
use regex::Regex;
use serenity::{
    all::{ChannelId, GuildId, UserId},
    prelude::*,
};
use std::collections::HashMap;

//the first few functions are public and used in main.rs
pub async fn resolve_mentions(guild_id: GuildId, message: &str) -> std::string::String {
    // Define the regex pattern
    let re = Regex::new(r"@(\S+)").unwrap();

    // Create a mutable String to hold the updated message
    let mut updated_message = message.to_string();
    let context_pre_init: &tokio::sync::RwLockReadGuard<Option<Context>> = &CONTEXT.read().await;
    let cur_context = context_pre_init.as_ref().unwrap();
    // Create a HashMap to cache the username-to-ID mappings
    let mut user_id_map: HashMap<String, UserId> = HashMap::new();

    // Iterate over all matches of the regex pattern in the message
    for cap in re.captures_iter(message) {
        if let Some(username) = cap.get(1) {
            let username = username.as_str().to_string();

            // Check if we've already queried this username
            if !user_id_map.contains_key(&username) {
                // Fetch the user by username
                let guild_test = cur_context.http.get_guild(guild_id).await.unwrap();

                for member in guild_test
                    .members(cur_context.http.clone(), None, None)
                    .await
                    .unwrap()
                {
                    if member.user.name == username {
                        user_id_map.insert(username.clone(), member.user.id);
                        break;
                    }
                }
            }

            // Replace the @username with <@UID_HERE>
            if let Some(user_id) = user_id_map.get(&username) {
                let mention = format!("<@{user_id}>");
                updated_message = updated_message.replace(&format!("@{username}"), &mention);
            }
        }
    }

    updated_message
}
pub async fn resolve_channels(guild_id: GuildId, message: &str) -> std::string::String {
    // Define the regex pattern
    let re = Regex::new(r"#(\S+)").unwrap();

    // Create a mutable String to hold the updated message
    let mut updated_message = message.to_string();
    let context_pre_init: &tokio::sync::RwLockReadGuard<Option<Context>> = &CONTEXT.read().await;
    let cur_context = context_pre_init.as_ref().unwrap();
    // Create a HashMap to cache the channel-to-ID mappings
    let mut channel_id_map: HashMap<String, ChannelId> = HashMap::new();

    // Iterate over all matches of the regex pattern in the message
    for cap in re.captures_iter(message) {
        if let Some(check_channel) = cap.get(1) {
            let check_channel = check_channel.as_str().to_string();

            // Check if we've already queried this channel name
            if !channel_id_map.contains_key(&check_channel) {
                // Fetch the channelid by channel name
                let guild_test = cur_context.http.get_guild(guild_id).await.unwrap();

                for channel_vars in guild_test.channels(cur_context.http.clone()).await.unwrap() {
                    let (channel_id, guild_channel) = channel_vars;
                    if guild_channel.name == check_channel {
                        channel_id_map.insert(check_channel.clone(), channel_id);
                        break;
                    }
                }
            }

            // Replace the @username with <@UID_HERE>
            if let Some(channel_id) = channel_id_map.get(&check_channel) {
                let mention = format!("<#{channel_id}>");
                updated_message = updated_message.replace(&format!("#{check_channel}"), &mention);
            }
        }
    }

    updated_message
}

pub async fn resolve_emojis(guild_id: GuildId, my_message: String) -> String{
    let pattern = r":([^:]+):";
    let result = modify_encapsulated_text(&my_message, pattern, guild_id).await;

    let my_message_temp = remove_colons_outside_angle_brackets(&result);
        
    // Regex to match !(!foo!)! where foo is any sequence of non-whitespace characters
    let re = Regex::new(r"!\(!(\S+?)!\)!").unwrap();

        // Replace matches with :foo:
    let result = re.replace_all(&my_message_temp, ":$1:");
    
    result.to_string()
}

//the following functions are used by the above functions and shouldn't be pub
async fn get_emoji_text(guild_id: GuildId, emoji_name: &str) -> Option<String> {
    let context_pre_init = &CONTEXT.read().await;
    let cur_context = context_pre_init.as_ref().unwrap();
    // Fetch the emojis in the guild
    if let Ok(emojis) = guild_id.emojis(&cur_context.http).await {
        // Search for the emoji with the given name
        for emoji in emojis {
            if emoji.name == emoji_name {
                // Return the formatted emoji text
                let emoji_text = format!("<:{}:{}>", emoji.name, emoji.id);
                return Some(emoji_text);
            }
        }
        None
    } else {
        message_top!("Couldn't connect!");
        None
    }
}



fn find_matches(text: &str, pattern: &str) -> Vec<(usize, usize, String)> {
    let re = match Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => panic!("bad regex {e}"),
    };

    // Collect all matches with their start and end positions
    re.captures_iter(text)
        .map(|caps| {
            let start = caps.get(0).unwrap().start();
            let end = caps.get(0).unwrap().end();
            let inner_text = caps.get(1).map_or("", |m| m.as_str()).to_string();
            (start, end, inner_text)
        })
        .collect()
}

async fn modify_encapsulated_text(text: &str, pattern: &str, guild_id: GuildId) -> String {
    let matches = find_matches(text, pattern);

    // Create a map of modifications
    let mut modifications = HashMap::new();
    for (_, _, emoji_text) in &matches {
        //need to get guild id
        let modified_text = get_emoji_text(guild_id, emoji_text)
            .await
            .unwrap_or_else(|| format!("!(!{emoji_text}!)!"));

        modifications.insert(emoji_text.clone(), modified_text);
    }

    // Rebuild the text with modifications, excluding the original markers
    let mut result = String::new();
    let mut last_index = 0;

    for (start, end, inner_text) in &matches {
        result.push_str(&text[last_index..*start]);
        result.push_str(&modifications[inner_text]);
        last_index = *end;
    }

    result.push_str(&text[last_index..]);
    result
}

fn remove_colons_outside_angle_brackets(input: &str) -> String {
    // Step 1: Find all text inside angle brackets and replace it with a placeholder
    let angle_bracket_re = Regex::new(r"<[^>]*>").unwrap();
    let mut placeholders = Vec::new();
    let mut temp_input = input.to_string();

    for (i, mat) in angle_bracket_re.find_iter(input).enumerate() {
        let placeholder = format!("@placeholder{i}");
        temp_input = temp_input.replace(mat.as_str(), &placeholder);
        placeholders.push((placeholder, mat.as_str()));
    }

    // Step 2: Remove colons around phrases not inside angle brackets
    let re = Regex::new(r":(\S+):").unwrap();
    let result = re.replace_all(&temp_input, "$1");

    // Step 3: Replace placeholders with original angle bracket content
    let mut final_result = result.to_string();
    for (placeholder, original) in placeholders {
        final_result = final_result.replace(&placeholder, original);
    }

    final_result
}