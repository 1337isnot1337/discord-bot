use crossterm::event::{Event, KeyCode};
use once_cell::sync::Lazy;
use ratatui::{
    style::{Style, Stylize},
    widgets::{block::Title, Block, List, ListItem},
};
use serenity::all::Message;
use tokio::sync::{Mutex, RwLock};
static LAST_MESSAGE_INFO: Lazy<Mutex<String>> = Lazy::new(|| {
    let var = String::new();
    var.into()
});
pub(crate) mod stdin {
    use crate::cleanup;
    use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
    use std::sync::OnceLock;
    use tokio::{
        spawn,
        sync::{
            mpsc::{channel, Receiver},
            RwLock,
        },
    };

    static STDIN: OnceLock<RwLock<Receiver<Event>>> = OnceLock::new();

    use std::time::Duration;

    pub(crate) fn init() {
        let (input_sender, input) = channel::<Event>(100);
        STDIN
            .set(RwLock::new(input))
            .expect("Failed to initialize STDIN");
        spawn(async move {
            loop {
                // Poll with a timeout to avoid blocking the entire loop
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    if let Ok(event) = event::read() {
                        if let Event::Key(KeyEvent {
                            code: KeyCode::Char('c'),
                            modifiers,
                            ..
                        }) = event
                        {
                            if modifiers.contains(KeyModifiers::CONTROL) {
                                cleanup();
                            }
                        }
                        input_sender
                            .send(event)
                            .await
                            .expect("Failed to send event");
                    }
                }
                // Optional: yield to avoid blocking the tokio runtime
                tokio::task::yield_now().await;
            }
        });
    }

    pub(super) async fn read() -> Event {
        STDIN
            .get()
            .expect("STDIN not initialized")
            .write()
            .await
            .recv()
            .await
            .expect("Failed to receive event")
    }
}

pub mod screen {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    };
    use once_cell::sync::Lazy;
    use ratatui::{
        layout::{Constraint, Direction, Layout, Rect},
        prelude::CrosstermBackend,
        style::{Style, Stylize},
        widgets::{Block, List},
        Terminal,
    };
    use std::{io, iter::once, string::String};
    use tokio::sync::RwLock;

    use super::{list, LAST_MESSAGE_INFO, TOP_MSG};

    static TERMINAL: Lazy<RwLock<Terminal<CrosstermBackend<io::Stdout>>>> = Lazy::new(|| {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).expect("Failed to initialize terminal");
        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )
        .expect("Failed to execute terminal commands");
        RwLock::new(terminal)
    });

    pub fn cleanup_term() {
        let mut terminal = TERMINAL.try_write().expect("Failed to lock terminal");
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .expect("Failed to execute terminal commands");
        terminal.show_cursor().expect("Failed to show cursor");
    }

    static LAYOUT: Lazy<RwLock<Layout>> = Lazy::new(|| {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .into()
    });

    pub async fn set_top(msg: &[String]) {
        let chunks = LAYOUT.read().await;

        TERMINAL
            .write()
            .await
            .draw(|f| {
                let chunk = chunks.split(f.size())[0];

                // Limit the number of lines displayed based on terminal height
                let height = chunk.height as usize - 1;
                let msg = &msg[msg.len().saturating_sub(height)..msg.len()];

                let top_messages = List::new(msg.iter().map(String::as_str))
                    .block(Block::bordered())
                    .style(Style::new().white().on_black());
                f.render_widget(top_messages, chunk);
            })
            .expect("Failed to draw top messages");
    }

    pub async fn set_stat_info(msg: String) {
        LAST_MESSAGE_INFO.lock().await.clone_from(&msg);
        let stat_messages_wid = list(once(msg), "Information".to_owned());
        let chunks = LAYOUT.read().await;
        let top_msg = TOP_MSG.read().await;
        let input_widget = list(once(""), "Enter text");
        let top_widg = list(top_msg.iter().map(String::as_str), "Top Mess");
        TERMINAL
            .write()
            .await
            .draw(|f| {
                let chunk = chunks.split(f.size());
                let input_rect = Rect::new(0, chunk[1].y, chunk[1].width / 2, chunk[1].height);
                let info_rect = Rect::new(
                    chunk[1].width / 2,
                    chunk[1].y,
                    chunk[1].width / 2,
                    chunk[1].height,
                );

                f.render_widget(input_widget, input_rect);
                f.render_widget(top_widg, chunk[0]);
                f.render_widget(stat_messages_wid, info_rect);
            })
            .expect("Failed to draw info messages");
    }

    pub async fn set_input(msg_to_send: &str, title: &str) {
        let foovar = LAST_MESSAGE_INFO.lock().await;
        let stat_messages_wid = list(once(&**foovar), "Information");
        let chunks = LAYOUT.read().await;
        let top_msg = TOP_MSG.read().await;
        let input_widget = list(once(msg_to_send), title);
        let top_widg = list(top_msg.iter().map(String::as_str), "Top Mess");
        TERMINAL
            .write()
            .await
            .draw(|f| {
                let chunk = chunks.split(f.size());
                let input_rect = Rect::new(0, chunk[1].y, chunk[1].width / 2, chunk[1].height);
                let info_rect = Rect::new(
                    chunk[1].width / 2,
                    chunk[1].y,
                    chunk[1].width / 2,
                    chunk[1].height,
                );
                f.render_widget(input_widget, input_rect);
                f.render_widget(top_widg, chunk[0]);
                f.render_widget(stat_messages_wid, info_rect);
            })
            .expect("Failed to draw input messages");
    }
}

static TOP_MSG: Lazy<RwLock<Vec<String>>> = Lazy::new(|| RwLock::new(Vec::new()));

pub async fn push_top(msg: String) {
    let mut all = TOP_MSG.write().await;
    all.push(msg);
    screen::set_top(&all).await;
}

#[macro_export]
macro_rules! message_top {
    () => {
        use $crate::local_ratatui::message_top_func;
        message_top_func("").await;
    };
    ($($arg:tt)*) => {{
        use $crate::local_ratatui::message_top_func;
        message_top_func(&format!($($arg)*)).await;
    }};
}

pub async fn message_top_func(msg: &str) {
    push_top(msg.to_string()).await;
}

fn list<'a, L: IntoIterator<Item = impl Into<ListItem<'a>>>, T: Into<Title<'a>>>(
    items: L,
    title: T,
) -> List<'a> {
    List::new(items)
        .block(Block::bordered().title(title))
        .style(Style::new().white().on_black())
}

pub async fn get_input(title: &str) -> String {
    let mut input = String::new();
    loop {
        input.push('█');
        screen::set_input(&input, title).await;
        input.pop();

        if let Event::Key(key) = stdin::read().await {
            match key.code {
                KeyCode::Enter if !input.is_empty() => break,
                KeyCode::Char(c) => input.push(c),
                KeyCode::Backspace => {
                    input.pop();
                }
                _ => {}
            }
        }
    }

    {
        let mut all = TOP_MSG.write().await;
        let msg = all.pop();
        screen::set_top(&all).await;
        msg
    };
    push_top(format!("{input}\n")).await;

    input
}

pub fn yap_about_user(discord_info: &Message) -> String {
    let edited_timestamp = discord_info
        .edited_timestamp
        .map_or_else(|| "Not edited".to_string(), |ts| ts.to_string());

    format!(
        "Author: {}, Message ID: {}, 
        Channel ID: {} \n, Timestamp: {}.\nContent: `{}`\n\
        Edited Timestamp: {}, TTS: {}, Mentions Everyone: {}\n, Mentions: {:?}, \
        Mentioned Roles: {:?}, Mentioned Channels: {:?}, Attachments: {:?}\n, \
        Embeds: {:?}, Reactions: {:?}\n, Pinned: {}, Webhook: {:?}, Message Kind: {:?}, \
        Activity: {:?}\n, Application: {:?}, Application ID: {:?}, \
        Referenced Message: {:?}, Interaction\n: {:?}, Thread: {:?}, Components: {:?}, \
        Member Info: {:?}",
        discord_info.author.name,
        discord_info.id,
        discord_info.channel_id,
        discord_info.timestamp,
        discord_info.content,
        edited_timestamp,
        discord_info.tts,
        discord_info.mention_everyone,
        discord_info.mentions,
        discord_info.mention_roles,
        discord_info.mention_channels,
        discord_info.attachments,
        discord_info.embeds,
        discord_info.reactions,
        discord_info.pinned,
        discord_info.webhook_id,
        discord_info.kind,
        discord_info.activity,
        discord_info.application,
        discord_info.application_id,
        discord_info.referenced_message,
        discord_info.interaction,
        discord_info.thread,
        discord_info.components,
        discord_info.member,
    )
}
