use iced::{
    widget::{button, column, container, row, text},
    Alignment, Element, Length, Subscription, Task, Theme,
};
use ollama_rs::models::LocalModel;

use crate::chat;

use crate::screen;
use crate::Screen;

#[derive(Debug, Clone)]
pub struct ModelSelect {
    loading_models: bool,
    local_models: Vec<LocalModel>,
}

#[derive(Debug, Clone)]
pub enum Message {
    GoToChatView(String),
    LocalModelsLoaded(Vec<LocalModel>),
}

pub enum Action {
    None,
    Run(Task<Message>),
    ChangeView(Screen),
}

impl ModelSelect {
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::GoToChatView(model) => {
                let chat_view = Screen::ChatView(screen::chat_view::ChatView::with_model(&model));
                Action::ChangeView(chat_view)
            }
            Message::LocalModelsLoaded(local_models) => {
                self.local_models = local_models;
                self.loading_models = false;
                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        if self.loading_models {
            let text = text("Loading models...");
            return container(text).center(Length::Fill).into();
        }

        let models: Vec<Element<'_, Message>> = self
            .local_models
            .iter()
            .map(|model| {
                let model_size = format!("{:.2} GB", (model.size as f64 / 1_000_000_000.0));

                let row = row![
                    text(&model.name).width(Length::Fill),
                    text(model_size).width(Length::Fill).align_x(Alignment::End)
                ]
                .padding(5);

                button(row)
                    .style(move |theme: &Theme, _| {
                        let palette = theme.extended_palette();
                        let mut style = button::Style::default();
                        let border = style.border;
                        let background = palette.background.weak;

                        style.text_color = background.text;
                        style.border = border.rounded(8);

                        style.with_background(background.color)
                    })
                    .on_press(Message::GoToChatView(model.name.clone()))
                    .into()
            })
            .collect();

        let column = column(models).spacing(10);

        container(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Alignment::Start)
            .padding(20)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

impl Default for ModelSelect {
    fn default() -> Self {
        let _ = Task::perform(chat::get_local_models(), Message::LocalModelsLoaded);
        Self {
            local_models: Vec::new(),
            loading_models: true,
        }
    }
}
