use std::sync::{Arc, Mutex};

use iced::futures::{
    channel::mpsc,
    stream::{Stream, StreamExt},
    SinkExt,
};
use ollama_rs::{
    generation::chat::{request::ChatMessageRequest, ChatMessage, ChatMessageResponse},
    models::LocalModel,
    Ollama,
};

#[derive(Debug, Clone)]
pub enum Event {
    Ready(mpsc::Sender<Message>),
    MessageGenerationStarted,
    MessageGenerationProgress(ChatMessageResponse),
    MessageGenerationEnded,
}

pub enum State {
    NotReady,
    Ready {
        ollama: Ollama,
        receiver: mpsc::Receiver<Message>,
    },
}

#[derive(Debug)]
pub enum Message {
    Generate {
        prompt: String,
        history: Vec<ChatMessage>,
        model: String,
    },
}

pub fn start() -> impl Stream<Item = Event> {
    iced::stream::channel(256, |mut output| async move {
        let mut state = State::NotReady;

        loop {
            match &mut state {
                State::NotReady => {
                    let (sender, receiver) = mpsc::channel(256);

                    let ollama = Ollama::default();
                    state = State::Ready { ollama, receiver };

                    let _ = output.send(Event::Ready(sender)).await;
                }
                State::Ready { ollama, receiver } => {
                    let message = receiver.select_next_some().await;
                    match message {
                        Message::Generate {
                            prompt,
                            history,
                            model,
                        } => {
                            let _ = output.send(Event::MessageGenerationStarted).await;

                            generate(&ollama, &mut output, history, prompt, model).await;

                            let _ = output.send(Event::MessageGenerationEnded).await;
                        }
                    }
                }
            }
        }
    })
}

pub async fn generate(
    ollama: &Ollama,
    sender: &mut mpsc::Sender<Event>,
    history: Vec<ChatMessage>,
    prompt: String,
    model: String,
) {
    let history = Arc::new(Mutex::new(history));

    let mut stream = ollama
        .send_chat_messages_with_history_stream(
            history,
            ChatMessageRequest::new(model, vec![ChatMessage::user(prompt)]),
        )
        .await
        .expect("Failed to create chat stream");

    while let Some(result) = stream.next().await {
        let response = result.expect("Something went wrong with the response");
        let _ = sender
            .send(Event::MessageGenerationProgress(response))
            .await;
    }
}

pub async fn get_local_models() -> Vec<LocalModel> {
    let ollama = Ollama::default();
    ollama
        .list_local_models()
        .await
        .unwrap_or_else(|_| Vec::new())
}
