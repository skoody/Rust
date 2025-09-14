//! The graphical user interface for the AI assistant.

use iced::widget::{button, column, text, text_input, scrollable, container};
use iced::{executor, Application, Command, Element, Settings, Length, Theme};

// --- To enable the real model, you'll need these imports ---
// use llm::models::Llama;
// use std::convert::Infallible;
// use std::path::PathBuf;

pub fn run_gui() -> iced::Result {
    GuiState::run(Settings::default())
}

// The state of our application
struct GuiState {
    input_value: String,
    conversation: Vec<String>,
    is_loading: bool,
    // --- To use a real model, you would store it here ---
    // model_session: Option<llm::InferenceSession>,
    // model: Option<Llama>,
}

// The messages our application will react to
#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    Submit,
    // --- Messages for real model interaction (uncomment to use) ---
    // ModelLoaded(Result<(Llama, llm::InferenceSession), String>),
    // TokenGenerated(String),
}

impl Application for GuiState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let initial_state = Self {
            input_value: String::new(),
            conversation: vec!["AI: Hello! How can I help you today? (mocked)".to_string()],
            is_loading: false,
            // model_session: None,
            // model: None,
        };

        (initial_state, Command::none())
    }

    fn title(&self) -> String {
        String::from("Local AI Assistant")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::Submit => {
                if !self.input_value.is_empty() && !self.is_loading {
                    let user_text = self.input_value.clone();
                    self.conversation.push(format!("You: {}", user_text));
                    self.input_value.clear();

                    // --- Mocked AI Response ---
                    self.is_loading = true;
                    let ai_response = format!("I am a mocked AI. I received: '{}'", user_text);
                    self.conversation.push(format!("AI: {}", ai_response));
                    self.is_loading = false;
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let conversation_view = scrollable(
            self.conversation.iter().fold(column![].spacing(10), |col, msg| {
                col.push(text(msg))
            })
        );

        let status_text = if self.is_loading { "AI is thinking..." } else { "Type your message..." };
        let input_view = text_input(status_text, &self.input_value)
            .on_input(Message::InputChanged)
            .on_submit(Message::Submit);

        let mut submit_button = button("Submit");
        if !self.is_loading {
            submit_button = submit_button.on_press(Message::Submit);
        }

        let content = column![
            container(conversation_view).height(Length::Fill),
            input_view,
            submit_button,
        ]
        .spacing(10)
        .padding(20);

        container(content).width(Length::Fill).height(Length::Fill).center_x().center_y().into()
    }
}
