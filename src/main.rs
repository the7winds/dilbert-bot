use std::time::Duration;

use teloxide::prelude::{
    AutoSend, ChosenInlineResult, Dispatcher, DispatcherHandlerRx, InlineQuery, Message, OnError,
    RequesterExt, StreamExt, UpdateWithCx,
};
use teloxide::requests::Requester;
use teloxide::types::{InlineQueryResult, InlineQueryResultPhoto};
use teloxide::utils::command::BotCommand;
use teloxide::Bot;
use tokio_stream::wrappers::UnboundedReceiverStream;

mod dilbert_search;

const BOTNAME: &'static str = "dilbert";

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum DilbertCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "search an image.")]
    Search(String),
}

async fn process_message(cx: UpdateWithCx<AutoSend<Bot>, Message>) -> anyhow::Result<()> {
    match cx.update.text() {
        None => Ok(()),
        Some(message) => {
            let command: DilbertCommand = DilbertCommand::parse(message, BOTNAME)?;

            match command {
                DilbertCommand::Help => {
                    cx.answer(DilbertCommand::descriptions()).await?;
                }
                DilbertCommand::Search(request) => {
                    let search_results = dilbert_search::search_image(request.as_str()).await?;
                    if search_results.is_empty() {
                        cx.answer("Nothing to be found.").await?;
                    } else {
                        for result in search_results.iter() {
                            cx.answer(result.page.as_str()).await?;
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

async fn message_handler(rx: DispatcherHandlerRx<AutoSend<Bot>, Message>) {
    UnboundedReceiverStream::new(rx)
        .for_each_concurrent(None, |cx| async move {
            process_message(cx).await.log_on_error().await;
        })
        .await;
}

async fn process_inline_query(cx: UpdateWithCx<AutoSend<Bot>, InlineQuery>) -> anyhow::Result<()> {
    let search_results = dilbert_search::search_image(cx.update.query.as_str()).await?;
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
                    caption: None,
                    parse_mode: None,
                    caption_entities: None,
                    reply_markup: None,
                    input_message_content: None,
                })
            }),
        )
        .await?;
    Ok(())
}

async fn inline_queries_handler(rx: DispatcherHandlerRx<AutoSend<Bot>, InlineQuery>) {
    UnboundedReceiverStream::new(rx)
        .for_each_concurrent(None, |cx| async move {
            process_inline_query(cx).await.log_on_error().await;
        })
        .await;
}

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting dilbert...");

    let bot = Bot::from_env().auto_send();
    Dispatcher::new(bot)
        .messages_handler(message_handler)
        .inline_queries_handler(inline_queries_handler)
        .dispatch()
        .await;
}
