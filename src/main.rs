pub mod chat;
pub mod screen;

use screen::{chat_view, model_select};

use iced::{Element, Subscription, Task, Theme};

fn main() -> iced::Result {
    iced::application("Ollama GUI", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .centered()
        .run_with(App::new)
}

struct App {
    screen: Screen,
}

#[derive(Debug, Clone)]
enum Message {
    ChatView(chat_view::Message),
    ModelSelect(model_select::Message),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                screen: Screen::ModelSelect(screen::ModelSelect::default()),
            },
            Task::future(chat::get_local_models())
                .map(model_select::Message::LocalModelsLoaded)
                .map(Message::ModelSelect),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ChatView(message) => {
                let Screen::ChatView(chat_view) = &mut self.screen else {
                    return Task::none();
                };
                let action = chat_view.update(message);
                match action {
                    chat_view::Action::None => Task::none(),
                    chat_view::Action::Run(task) => task.map(Message::ChatView),
                    chat_view::Action::ChangeView(screen) => {
                        self.screen = screen;
                        if let Screen::ModelSelect(_) = &self.screen {
                            return Task::future(chat::get_local_models())
                                .map(model_select::Message::LocalModelsLoaded)
                                .map(Message::ModelSelect);
                        };

                        Task::none()
                    }
                }
            }
            Message::ModelSelect(message) => {
                let Screen::ModelSelect(model_select) = &mut self.screen else {
                    return Task::none();
                };
                let action = model_select.update(message);
                match action {
                    model_select::Action::None => Task::none(),
                    model_select::Action::Run(task) => task.map(Message::ModelSelect),
                    model_select::Action::ChangeView(screen) => {
                        self.screen = screen;
                        Task::none()
                    }
                }
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::ChatView(chat_view) => chat_view.view().map(Message::ChatView),
            Screen::ModelSelect(model_select) => model_select.view().map(Message::ModelSelect),
        }
    }

    fn theme(&self) -> Theme {
        Theme::GruvboxDark
    }

    fn subscription(&self) -> Subscription<Message> {
        let subscription = match &self.screen {
            Screen::ChatView(chat_view) => chat_view.subscription().map(Message::ChatView),
            Screen::ModelSelect(model_select) => {
                model_select.subscription().map(Message::ModelSelect)
            }
        };

        subscription
    }
}

#[derive(Debug, Clone)]
pub enum Screen {
    ChatView(screen::ChatView),
    ModelSelect(screen::ModelSelect),
}
