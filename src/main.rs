use std::sync::Arc;
use std::time::Duration;

use crate::dilbert::search::SearchSettings;
use teloxide::prelude::{
    AutoSend, Dispatcher, DispatcherHandlerRx, InlineQuery, Message, OnError, RequesterExt,
    StreamExt, UpdateWithCx,
};
use teloxide::requests::Requester;
use teloxide::types::{InlineQueryResult, InlineQueryResultPhoto};
use teloxide::utils::command::BotCommand;
use teloxide::Bot;
use tokio_stream::wrappers::UnboundedReceiverStream;

mod dilbert;

const BOTNAME: &'static str = "dilbert";

#[derive(BotCommand, Debug)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum DilbertCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "search an image.")]
    Search(String),
}

async fn process_message(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    settings: &SearchSettings,
) -> anyhow::Result<()> {
    match cx.update.text() {
        None => Ok(()),
        Some(message) => {
            let command: DilbertCommand = DilbertCommand::parse(message, BOTNAME)?;
            log::info!("Got a new command: {:?}", command);

            match command {
                DilbertCommand::Help => {
                    cx.answer(DilbertCommand::descriptions()).await?;
                    log::info!("Send help info.");
                }
                DilbertCommand::Search(request) => {
                    let search_results =
                        dilbert::search::search_image(request.as_str(), settings).await?;
                    if search_results.is_empty() {
                        cx.answer("Nothing to be found.").await?;
                        log::info!("Nothing to be found.");
                    } else {
                        for result in search_results.iter() {
                            cx.answer(result.page.as_str()).await?;
                            log::info!("Send a response: {}", result.page);
                            // FIXME: find a better solution.
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }
            }
            Ok(())
        }
    }
}

async fn message_handler(
    rx: DispatcherHandlerRx<AutoSend<Bot>, Message>,
    settings: &SearchSettings,
) {
    UnboundedReceiverStream::new(rx)
        .for_each_concurrent(None, |cx| async move {
            process_message(cx, settings).await.log_on_error().await;
        })
        .await;
}

async fn process_inline_query(
    cx: UpdateWithCx<AutoSend<Bot>, InlineQuery>,
    settings: &SearchSettings,
) -> anyhow::Result<()> {
    log::info!("Has inline query.");
    let search_results = dilbert::search::search_image(cx.update.query.as_str(), settings).await?;
    cx.requester
        .answer_inline_query(
            cx.update.id,
            search_results.iter().map(|r| {
                InlineQueryResult::Photo(InlineQueryResultPhoto {
                    id: r.image.to_string(),
                    photo_url: r.image.to_string(),
                    thumb_url: r.image.to_string(),
                    photo_width: None,
                    photo_height: None,
                    title: None,
                    description: None,
                    caption: Some(format!("source: {}", r.page).to_string()),
                    parse_mode: None,
                    caption_entities: None,
                    reply_markup: None,
                    input_message_content: None,
                })
            }),
        )
        .await?;
    log::info!("Send {} URLs in response.", search_results.len());
    Ok(())
}

async fn inline_queries_handler(
    rx: DispatcherHandlerRx<AutoSend<Bot>, InlineQuery>,
    settings: &SearchSettings,
) {
    UnboundedReceiverStream::new(rx)
        .for_each_concurrent(None, |cx| async move {
            process_inline_query(cx, settings)
                .await
                .log_on_error()
                .await;
        })
        .await;
}

async fn run() {
    env_logger::init();
    log::info!("Starting dilbert...");

    let bot = Bot::from_env().auto_send();

    let search_settings = Arc::new(SearchSettings::from_env());

    let dispatcher = Dispatcher::new(bot);
    let dispatcher = {
        let search_settings = search_settings.clone();
        dispatcher.messages_handler(|rx| async move {
            message_handler(rx, Arc::clone(&search_settings).as_ref()).await
        })
    };
    let mut dispatcher = {
        let search_settings = search_settings.clone();
        dispatcher.inline_queries_handler(|rx| async move {
            inline_queries_handler(rx, search_settings.as_ref()).await
        })
    };
    dispatcher.dispatch().await;
}

#[tokio::main]
async fn main() {
    // According to teloxide, it's better to split main because of tokio
    run().await;
}
