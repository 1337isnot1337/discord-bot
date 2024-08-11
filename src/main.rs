use crate::local_ratatui::get_input;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use local_ratatui::{
    message_top_func,
    screen::{self, cleanup_term},
    stdin, yap_about_user,
};
use once_cell::sync::Lazy;
use serenity::{all::ChannelId, async_trait, model::channel::Message, prelude::*};
use std::{
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

static RESPOND_BACK: Lazy<tokio::sync::RwLock<bool>> =
    Lazy::new(|| tokio::sync::RwLock::new(false));
static ALL_INTERACT: Lazy<tokio::sync::RwLock<bool>> =
    Lazy::new(|| tokio::sync::RwLock::new(false));
pub static PREVIOUS_INDEX: Lazy<tokio::sync::RwLock<usize>> =
    Lazy::new(|| tokio::sync::RwLock::new(0));

static FILE: Lazy<tokio::sync::RwLock<File>> = Lazy::new(|| {
    let file = if Path::new("messages.txt").exists() {
        OpenOptions::new()
            .write(true)
            .create(false)
            .truncate(false)
            .open("messages.txt")
            .unwrap()
    } else {
        File::create("messages.txt").unwrap()
    };
    tokio::sync::RwLock::new(file)
});
static CONTEXT: Lazy<RwLock<Option<Context>>> = Lazy::new(|| tokio::sync::RwLock::new(None));

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if CONTEXT.read().await.is_none() {
            *CONTEXT.write().await = Some(ctx.clone());
        }
        if msg.author.id == 1_102_939_243_954_847_745 || *ALL_INTERACT.read().await {
            screen::set_stat_info(yap_about_user(&msg)).await;

            for guild in ctx.cache.guilds() {
                message_top!("Guild ID: {guild}");
                let msg = if let Some(var) = ctx.cache.guild(guild) {
                    let name: String = var.name.clone();
                    format!("Name of guild: {name}")
                } else {
                    "Could not get name of guild!".to_owned()
                };
                message_top_func(&msg).await;
                match ctx.http.get_channels(guild).await {
                    Ok(channels) => {
                        for channel in channels {
                            message_top!("Channel ID: {}, Name: {}", channel.id, channel.name);
                        }
                    }
                    Err(e) => {
                        message_top!("Error fetching channels: {e:?}");
                    }
                }
            }
            let mut file = FILE.write().await;
            file.write_all(format!("{}\n", msg.content).as_bytes())
                .unwrap();
            drop(file);

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
                let path = "confess.txt";

                let mut file = OpenOptions::new().append(true).create(true).open(path).unwrap();
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
/*
 */

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
        let token = get_input().await;

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
            let foovar = &CONTEXT.read().await;
            let cur_context = foovar.as_ref().unwrap();
            let parsed: ChannelId;
            loop {
                message_top!("Enter channel id");
                let init_var = get_input().await;
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
            let my_message = get_input().await;

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

fn cleanup() {
    disable_raw_mode().unwrap();
    cleanup_term();
    print!("\x1B[?25h");
    io::stdout().flush().unwrap();
    clearscreen::clear().expect("Failed to clear screen");
    process::exit(0);
}
