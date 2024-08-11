use crate::local_ratatui::get_input;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use local_ratatui::{
    screen::{self, cleanup_term},
    stdin, yap_about_user,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::{
    all::{ChannelId, GuildId, UserId},
    async_trait,
    model::channel::Message,
    prelude::*,
};
use std::{
    collections::HashMap,
    env,
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{spawn, time::sleep};

mod local_ratatui;
static SENT_CHANNELS: Lazy<tokio::sync::RwLock<bool>> =
    Lazy::new(|| tokio::sync::RwLock::new(false));
static RESPOND_BACK: Lazy<tokio::sync::RwLock<bool>> =
    Lazy::new(|| tokio::sync::RwLock::new(false));
static ALL_INTERACT: Lazy<tokio::sync::RwLock<bool>> =
    Lazy::new(|| tokio::sync::RwLock::new(false));
pub static PREVIOUS_INDEX: Lazy<tokio::sync::RwLock<usize>> =
    Lazy::new(|| tokio::sync::RwLock::new(0));

static FILE: Lazy<tokio::sync::RwLock<File>> = Lazy::new(|| {
    let file = if Path::new("txt_files/messages.txt").exists() {
        OpenOptions::new()
            .write(true)
            .create(false)
            .truncate(false)
            .open("txt_files/messages.txt")
            .unwrap()
    } else {
        File::create("txt_files/messages.txt").unwrap()
    };
    tokio::sync::RwLock::new(file)
});
static CONTEXT: Lazy<RwLock<Option<Context>>> = Lazy::new(|| tokio::sync::RwLock::new(None));

struct Handler;
#[allow(clippy::too_many_lines)]
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if CONTEXT.read().await.is_none() {
            *CONTEXT.write().await = Some(ctx.clone());
        }
        if msg.author.id == 1_102_939_243_954_847_745 || *ALL_INTERACT.read().await {
            screen::set_stat_info(yap_about_user(&msg)).await;

            if !*SENT_CHANNELS.read().await {
                let mut channels_string = String::new();
                for guild in ctx.cache.guilds() {
                    message_top!("Guild ID: {guild}");
                    let msg = if let Some(var) = ctx.cache.guild(guild) {
                        let name: String = var.name.clone();
                        format!("Name of guild: {name}\n")
                    } else {
                        "Could not get name of guild!\n".to_owned()
                    };
                    channels_string.push_str(&msg);
                    match ctx.http.get_channels(guild).await {
                        Ok(channels) => {
                            for channel in channels {
                                channels_string.push_str(&format!(
                                    "\nChannel ID: {},\nName: {}\n",
                                    channel.id, channel.name
                                ));
                            }
                        }
                        Err(e) => {
                            message_top!("Error fetching channels: {e:?}");
                        }
                    }
                }

                let mut file = File::create("txt_files/channels.txt").unwrap();
                file.write_all(&channels_string.into_bytes()).unwrap();
                *SENT_CHANNELS.write().await = true;
            }
            let mut file = FILE.write().await;
            file.write_all(format!("{}\n", msg.content).as_bytes())
                .unwrap();
            drop(file);
            //following are more custom commands and such
            if msg.content == "crazy" {
                if let Err(why) = msg.channel_id.say(&ctx.http, "Crazy? I was crazy once. They locked me in a room. A rubber room. A rubber room with rats. The rats made me crazy.").await {
                    message_top!("Error sending message: {why:?}");
                }
            }
            if msg.content.contains("sigma") {
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, "https://tenor.com/nmfj36p1Tdl.gif")
                    .await
                {
                    message_top!("Error sending message: {why:?}");
                }
            }
            if msg.content.contains("/confess") {
                let revealed = format!("User {} tried to confess {}", msg.author.name, msg.content);
                let path = "txt_files/confess.txt";

                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(path)
                    .unwrap();
                file.write_all(revealed.as_bytes()).unwrap();
            }
            /*
            if msg.author.name == ".catcode" {
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, "https://tenor.com/flmtUwgYAqH.gif")
                    .await
                {
                    message_top!("Error sending message: {why:?}");
                }
            }
            */

            if msg.content == "!help" {
                if let Err(why) = msg
                    .channel_id
                    .say(
                        &ctx.http,
                        "
                Displaying Help messages for Owo-bot...

                Commands:
                \"crazy\" -- crazy meme
                \"!user_info\" -- displays user info from bot perspective
                \"--help\" -- displays this message
                \"sigma\" -- displays what the sigma gif
                \"/confess\" -- confess your sins!

                contact @t3rabit3 for more info


                ",
                    )
                    .await
                {
                    message_top!("Error sending message: {why:?}");
                }
            }

            if msg.content.contains("!user_info")
                && (msg.author.id != 1_271_514_040_786_747_495
                    && msg.author.id != 1_271_653_492_049_576_026)
            {
                if let Err(why) = msg.channel_id.say(&ctx.http, yap_about_user(&msg)).await {
                    message_top!("Error sending message: {why:?}");
                }
            }
        }
    }
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
fn replace_special_syntax(input: &str) -> String {
    // Regex to match !(!foo!)! where foo is any sequence of non-whitespace characters
    let re = Regex::new(r"!\(!(\S+?)!\)!").unwrap();

    // Replace matches with :foo:
    let result = re.replace_all(input, ":$1:");

    result.to_string()
    
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

static SECTION: Lazy<Mutex<()>> = Lazy::new(|| tokio::sync::Mutex::new(()));

#[tokio::main]
async fn main() {
    enable_raw_mode().unwrap();

    stdin::init();

    // Atomic boolean to track if SIGINT was received
    let running = Arc::new(AtomicBool::new(true));
    let _r = running.clone();

    while running.load(Ordering::SeqCst) {
        begin_events().await;
    }
}

async fn begin_events() {
    let args: Vec<String> = env::args().collect::<Vec<String>>()[1..].to_vec();
    let mut invalid_args = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--respond" => {
                *RESPOND_BACK.write().await = true;
                message_top!("You can respond back!");
            }
            "--anyone" => {
                *ALL_INTERACT.write().await = true;
                message_top!("Anyone can activate!");
            }
            _ => invalid_args.push(arg),
        }
    }

    if !invalid_args.is_empty() {
        let error_string = format!(
            "The following args were not recognized: {}",
            invalid_args.join(", ")
        );
        message_top!("{}", error_string);
        process::exit(0);
    }

    loop {
        message_top!("Enter your bot token: ");
        let token = get_input("Insert Token").await;

        message_top!("Token is {}", token);

        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(&token, intents)
            .event_handler(Handler)
            .await
            .expect("Err creating client");
        let _messaging_thread = spawn(async { message_func().await });
        if let Err(why) = client.start().await {
            message_top!("Client error: {why:?}");
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }
        break;
    }
}

async fn message_func() {
    while CONTEXT.read().await.is_none() {
        sleep(Duration::from_millis(10)).await;
    }
    loop {
        if *RESPOND_BACK.read().await {
            let _guard = SECTION.lock().await;
            let context_pre_init: &tokio::sync::RwLockReadGuard<Option<Context>> =
                &CONTEXT.read().await;
            let cur_context = context_pre_init.as_ref().unwrap();
            let parsed: ChannelId;
            loop {
                message_top!("Enter channel id");
                let init_var = get_input("Enter channel ID or Name").await;
                if let Ok(val) = init_var.parse() {
                    parsed = val;
                } else {
                    parsed = if let Some(id) =
                        find_channel_by_name_in_all_guilds(cur_context, init_var).await
                    {
                        id
                    } else {
                        message_top!("Invalid!");
                        continue;
                    };
                };

                message_top!("Enter message");

                break;
            }
            let guild_id: GuildId =
                if let Some(var) = find_guild_id_by_channel_id(cur_context, parsed).await {
                    var
                } else {
                    message_top!("Some error occured. Couldn't fetch guild id. Restarting.");
                    continue;
                };
            let mut my_message = get_input("Enter message").await;
            if my_message.contains(':') {
                let pattern = r":([^:]+):";
                let result = modify_encapsulated_text(&my_message, pattern, guild_id).await;

                let my_message_temp = remove_colons_outside_angle_brackets(&result);
                my_message = replace_special_syntax(&my_message_temp);
            }

            if my_message.contains('@') {
                my_message = resolve_mentions(guild_id, &my_message).await;
            }
            if my_message.contains('#') {
                my_message = resolve_channels(guild_id, &my_message).await;
            }

            if let Err(why) = parsed.say(&cur_context.http.clone(), my_message).await {
                message_top!("Error sending message: {why:?}");
            }
            message_top!("Message sent.");
            sleep(Duration::from_millis(10)).await;
        }
    }
}

async fn find_channel_by_name_in_all_guilds(
    ctx: &Context,
    channel_name: String,
) -> Option<ChannelId> {
    // Get a list of all guilds the bot is in
    let guilds = ctx.cache.guilds();

    // Iterate over each guild
    for guild_id in guilds {
        // Fetch all channels in the guild
        if let Ok(channels) = guild_id.channels(&ctx.http).await {
            // Iterate over the channels and look for a channel with the given name
            for (channel_id, channel) in channels {
                if channel.name == channel_name {
                    return Some(channel_id);
                }
            }
        }
    }

    None
}

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

fn cleanup() {
    disable_raw_mode().unwrap();
    cleanup_term();
    print!("\x1B[?25h");
    io::stdout().flush().unwrap();
    clearscreen::clear().expect("Failed to clear screen");
    process::exit(0);
}

async fn find_guild_id_by_channel_id(ctx: &Context, channel_id: ChannelId) -> Option<GuildId> {
    // Get the list of all guilds the bot is a member of
    let guilds = match ctx
        .http
        .get_guilds(
            Some(serenity::http::GuildPagination::After(GuildId::new(1))),
            Some(200),
        )
        .await
    {
        Ok(var) => var,
        Err(e) => panic!("{e}"),
    };

    // Iterate through each guild
    for guild_info in guilds {
        // Fetch the guild to get detailed information
        if let Ok(guild) = guild_info.id.to_partial_guild(&ctx.http).await {
            // Check if the channel ID exists within this guild
            if guild
                .channels(&ctx.http)
                .await
                .ok()
                .unwrap()
                .contains_key(&channel_id)
            {
                return Some(guild_info.id);
            }
        }
    }

    None
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

async fn resolve_mentions(guild_id: GuildId, message: &str) -> std::string::String {
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

async fn resolve_channels(guild_id: GuildId, message: &str) -> std::string::String {
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
