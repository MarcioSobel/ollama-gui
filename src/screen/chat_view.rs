use iced::{
    futures::channel::mpsc,
    widget::{button, column, container, row, scrollable, text, text_input},
    Alignment, Element, Length, Padding, Subscription, Task, Theme,
};
use ollama_rs::generation::chat::{ChatMessage, MessageRole};

use crate::{chat, Screen};

use super::model_select;

#[derive(Debug, Clone)]
pub struct ChatView {
    prompt: String,
    waiting_for_response: bool,
    history: Vec<ChatMessage>,
    state: State,
    model: String,
}

#[derive(Clone, Debug)]
pub enum Message {
    PromptContentChanged(String),
    SubmitPrompt,
    Chat(chat::Event),
    GoToModelSelect,
}

#[derive(Debug, Clone)]
enum State {
    Disconnected,
    Connected(mpsc::Sender<chat::Message>),
}

pub enum Action {
    None,
    Run(Task<Message>),
    ChangeView(Screen),
}

impl ChatView {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::PromptContentChanged(content) => {
                self.prompt = content;
                Action::None
            }
            Message::SubmitPrompt => match &mut self.state {
                State::Disconnected => Action::None,
                State::Connected(sender) => {
                    let prompt = self.prompt.to_string();
                    let model = self.model.to_string();

                    sender
                        .try_send(chat::Message::Generate {
                            model,
                            prompt,
                            history: self.history.clone(),
                        })
                        .expect("Failed to send message");

                    self.history.push(ChatMessage::user(self.prompt.clone()));

                    Action::None
                }
            },
            Message::Chat(event) => match event {
                chat::Event::Ready(sender) => {
                    self.state = State::Connected(sender);

                    Action::None
                }
                chat::Event::MessageGenerationStarted => {
                    self.waiting_for_response = true;
                    self.prompt = String::new();
                    self.history.push(ChatMessage::assistant(String::new()));

                    Action::None
                }
                chat::Event::MessageGenerationProgress(token) => {
                    let i = self.history.len() - 1;
                    self.history[i].content += &token.message.content;

                    Action::None
                }
                chat::Event::MessageGenerationEnded => {
                    self.waiting_for_response = false;

                    Action::None
                }
            },
            Message::GoToModelSelect => {
                let model_select = Screen::ModelSelect(model_select::ModelSelect::default());
                Action::ChangeView(model_select)
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![self.header(), self.content(), self.footer()]
            .spacing(10)
            .padding(20)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(chat::start).map(Message::Chat)
    }

    fn header(&self) -> Element<'_, Message> {
        let back_button = button("< Go back").on_press(Message::GoToModelSelect);

        let title = text(&self.model)
            .size(16)
            .width(Length::Fill)
            .align_x(Alignment::Center);

        let size = text(format!("{} messages", self.history.len())).size(14);

        row![back_button, title, size].spacing(10).into()
    }

    fn footer(&self) -> Element<'_, Message> {
        let send_button = button("Send").on_press(Message::SubmitPrompt);
        let input_box = text_input("Input", &self.prompt)
            .on_input_maybe(if self.waiting_for_response {
                None
            } else {
                Some(Message::PromptContentChanged)
            })
            .on_submit_maybe(if self.waiting_for_response {
                None
            } else {
                Some(Message::SubmitPrompt)
            })
            .width(Length::Fill);

        row![input_box, send_button]
            .spacing(10)
            .width(Length::Fill)
            .into()
    }

    fn content(&self) -> Element<'_, Message> {
        let chat_messages: Vec<Element<'_, Message>> = self
            .history
            .iter()
            .map(|chat_message| Self::chat_message(chat_message.clone()))
            .collect();

        let columns = column(chat_messages).spacing(10);

        scrollable(
            container(columns)
                .width(Length::Fill)
                .align_y(Alignment::Start)
                .padding(Padding::default().right(5).left(5)),
        )
        .height(Length::Fill)
        .into()
    }

    fn chat_message(message: ChatMessage) -> Element<'static, Message> {
        let content = message.content.clone();
        let padding = Padding::default().bottom(5).top(7).left(10).right(10);

        let alignment = match message.role {
            MessageRole::User => Alignment::End,
            MessageRole::Assistant => Alignment::Start,
            MessageRole::System => Alignment::Center,
            MessageRole::Tool => Alignment::Center,
        };

        let chat_message = container(text(content).size(20))
            .max_width(600.0)
            .padding(padding)
            .style(move |theme: &Theme| {
                let palette = theme.extended_palette();
                let border = container::Style::default().border;

                let background = match message.role {
                    MessageRole::User => palette.background.weak,
                    MessageRole::Assistant => palette.background.strong,
                    MessageRole::System => palette.background.base,
                    MessageRole::Tool => palette.background.base,
                };

                container::Style::default()
                    .background(background.color)
                    .color(background.text)
                    .border(border.rounded(8))
            });

        container(chat_message)
            .align_x(alignment)
            .width(Length::Fill)
            .into()
    }

    pub fn with_model(model: &str) -> Self {
        Self {
            model: model.to_string(),
            ..Default::default()
        }
    }
}

impl Default for ChatView {
    fn default() -> Self {
        Self {
            waiting_for_response: false,
            prompt: String::new(),
            history: Vec::new(),
            state: State::Disconnected,
            model: String::new(),
        }
    }
}
