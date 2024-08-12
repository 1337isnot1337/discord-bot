use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use once_cell::sync::Lazy;
use serenity::{
    all::{ChannelId, GuildId, ReactionType},
    async_trait,
    model::channel::Message,
    prelude::*,
};
use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Write},
    panic,
    path::Path,
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{spawn, time::sleep};

use local_ratatui::{
    get_input,
    screen::{self, cleanup_term},
    stdin, yap_about_user,
};
use patterns::{resolve_channels, resolve_emojis, resolve_mentions};
use yapping::yapping;

mod local_ratatui;
mod patterns;
mod yapping;

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

static YOUR_LAST_MESSAGE: Lazy<RwLock<Option<ChannelId>>> =
    Lazy::new(|| tokio::sync::RwLock::new(None));
static THE_LAST_MESSAGE: Lazy<RwLock<Option<ChannelId>>> =
    Lazy::new(|| tokio::sync::RwLock::new(None));

struct Handler;
#[allow(clippy::too_many_lines)]
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if CONTEXT.read().await.is_none() {
            *CONTEXT.write().await = Some(ctx.clone());
        }
        *THE_LAST_MESSAGE.write().await = Some(msg.channel_id);
        if (msg.author.id != 1_271_514_040_786_747_495)
            && (msg.author.id != 1_271_653_492_049_576_026)
            && (msg.author.id != 810_609_798_268_715_029)
        {
            store(&msg);
        };

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

            if msg.content.contains("sigma")
                && (msg.author.id != 1_271_514_040_786_747_495)
                && (msg.author.id != 1_271_653_492_049_576_026)
            {
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
                \"!yap\" -- the bot will yap

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
            if msg.content == "!yap"
                && (msg.author.id != 1_271_514_040_786_747_495
                    && msg.author.id != 1_271_653_492_049_576_026)
            {
                if let Err(why) = msg.channel_id.say(&ctx.http, yapping()).await {
                    message_top!("Error sending message: {why:?}");
                }
            }
        }
        if msg.author.id == 1_252_795_706_188_496_906 {
            let emoji = ReactionType::Unicode("🤓".to_string());
            msg.react(ctx.http, emoji).await.unwrap();
        }
    }
}

static SECTION: Lazy<Mutex<()>> = Lazy::new(|| tokio::sync::Mutex::new(()));

#[tokio::main]
async fn main() {
    let default_hook = panic::take_hook();

    // Set the custom panic hook
    panic::set_hook(Box::new(move |info| {
        disable_raw_mode().unwrap();
        cleanup_term();
        print!("\x1B[?25h");
        io::stdout().flush().unwrap();
        clearscreen::clear().expect("Failed to clear screen");

        // Get panic location (file and line number)
        let location = info.location().map_or_else(
            || "unknown location".to_string(),
            |loc| format!("{}:{}", loc.file(), loc.line()),
        );

        // Get panic message
        let message = info.payload().downcast_ref::<&str>().unwrap_or(&"Box<Any>");

        // Log the panic details
        let _ = writeln!(
            io::stderr(),
            "Custom panic handler: Panic occurred at {location}: {message}"
        );

        // Call the default panic hook
        default_hook(info);

        // Exit the program with a non-zero code
        process::exit(1);
    }));

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
    //wait for a sufficient context
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
                if let Ok(id) = init_var.parse() {
                    parsed = id;
                } else {
                    parsed = if let Some(id) =
                        find_channel_by_name_in_all_guilds(cur_context, &init_var).await
                    {
                        id
                    } else if init_var.contains('!') {
                        match init_var.as_str() {
                            "!mylast" => {
                                if let Some(channel_id) = *YOUR_LAST_MESSAGE.read().await {
                                    channel_id
                                } else {
                                    message_top!("There is no previous message of yours cached!");
                                    continue;
                                }
                            }
                            "!thelast" => {
                                if let Some(channel_id) = *THE_LAST_MESSAGE.read().await {
                                    channel_id
                                } else {
                                    message_top!("There is no previous message at all cached!");
                                    continue;
                                }
                            }
                            _ => {
                                message_top!("Invalid ! command.");
                                continue;
                            }
                        }
                    } else {
                        message_top!("Invalid channel ID or channel command!");
                        continue;
                    };
                };

                message_top!("Enter message");

                break;
            }
            *YOUR_LAST_MESSAGE.write().await = Some(parsed);
            *THE_LAST_MESSAGE.write().await = Some(parsed);
            let guild_id: GuildId =
                if let Some(var) = find_guild_id_by_channel_id(cur_context, parsed).await {
                    var
                } else {
                    message_top!("Some error occured. Couldn't fetch guild id. Restarting.");
                    continue;
                };
            let mut my_message = get_input("Enter message").await;
            if my_message.contains(':') {
                my_message = resolve_emojis(guild_id, my_message).await;
            }

            if my_message.contains('@') {
                my_message = resolve_mentions(guild_id, &my_message).await;
            }
            if my_message.contains('#') {
                my_message = resolve_channels(guild_id, &my_message).await;
            }
            if my_message == "!yap" {
                my_message = yapping();
            }

            if let Err(why) = parsed.say(&cur_context.http.clone(), my_message).await {
                message_top!("Error sending message: {why:?}");
            }
            message_top!("Message sent.");
        }
    }
}

async fn find_channel_by_name_in_all_guilds(
    ctx: &Context,
    channel_name: &String,
) -> Option<ChannelId> {
    // Get a list of all guilds the bot is in
    let guilds = ctx.cache.guilds();

    // Iterate over each guild
    for guild_id in guilds {
        // Fetch all channels in the guild
        if let Ok(channels) = guild_id.channels(&ctx.http).await {
            // Iterate over the channels and look for a channel with the given name
            for (channel_id, channel) in channels {
                if channel.name == *channel_name {
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

fn store(msg: &Message) {
    let mut file = OpenOptions::new()
        .create(false)
        .truncate(false)
        .append(true)
        .open("txt_files/input.txt")
        .unwrap();
    let bad_words = [
        "fag",
        "nig",
        "higgers",
        "f@g",
        "!yap",
        "!work",
        "!slut",
        "!roulette",
        "!blackjack",
    ]; //add more bad words here if needed

    if !bad_words.iter().any(|&x| msg.content.contains(x)) {
        file.write_all((format!("{} ", msg.content)).as_bytes())
            .unwrap();
    }
}
