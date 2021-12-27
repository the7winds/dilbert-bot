use teloxide::prelude::{
    AutoSend, ChosenInlineResult, Dispatcher, DispatcherHandlerRx, InlineQuery, Message, OnError,
    RequesterExt, StreamExt, UpdateWithCx,
};
use teloxide::utils::command::BotCommand;
use teloxide::Bot;

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
                    cx.answer(format!("You have requested: {}", request))
                        .await?;
                }
            }
            Ok(())
        }
    }
}

async fn message_handler(rx: DispatcherHandlerRx<AutoSend<Bot>, Message>) {
    tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
        .for_each_concurrent(None, |cx| async move {
            process_message(cx).await.log_on_error().await;
        })
        .await;
}

async fn inline_queries_handler(_rx: DispatcherHandlerRx<AutoSend<Bot>, InlineQuery>) {}

async fn chosen_inline_results_handler(
    _rx: DispatcherHandlerRx<AutoSend<Bot>, ChosenInlineResult>,
) {
}

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting dilbert...");

    let bot = Bot::from_env().auto_send();
    Dispatcher::new(bot)
        .messages_handler(message_handler)
        .inline_queries_handler(inline_queries_handler)
        .chosen_inline_results_handler(chosen_inline_results_handler)
        .dispatch()
        .await;
}
