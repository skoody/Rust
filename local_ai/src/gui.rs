//! The graphical user interface for the AI assistant.

use iced::widget::{button, column, text, text_input, scrollable, container};
use iced::{Element, Sandbox, Settings, Length};

pub fn run_gui() -> iced::Result {
    GuiState::run(Settings::default())
}

#[derive(Default)]
struct GuiState {
    input_value: String,
    conversation: Vec<String>,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    Submit,
}

impl Sandbox for GuiState {
    type Message = Message;

    fn new() -> Self {
        Self {
            input_value: String::new(),
            conversation: vec!["AI: Hello! How can I help you today? (mocked)".to_string()],
        }
    }

    fn title(&self) -> String {
        String::from("Local AI Assistant")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(value) => {
                self.input_value = value;
            }
            Message::Submit => {
                if !self.input_value.is_empty() {
                    let user_text = self.input_value.clone();
                    self.conversation.push(format!("You: {}", user_text));

                    // --- Mocked AI Response ---
                    // In a real app, this would call the LLM and the command engine.
                    let ai_response = if user_text.eq_ignore_ascii_case("ls") {
                        // Simulate command execution
                        self.conversation.push("AI: !cmd:ls -l".to_string());
                        // In a real GUI, we'd need a way to show the command output.
                        // For now, we'll just show the command.
                        "Executing `ls -l`... (output would appear here)".to_string()
                    } else {
                        format!("I am a mocked AI. I received: '{}'", user_text)
                    };

                    self.conversation.push(format!("AI: {}", ai_response));
                    self.input_value.clear();
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let conversation_view = scrollable(
            self.conversation.iter().fold(column![].spacing(10), |col, msg| {
                col.push(text(msg))
            })
        );

        let input_view = text_input("Type your message...", &self.input_value)
            .on_input(Message::InputChanged)
            .on_submit(Message::Submit);

        let submit_button = button("Submit").on_press(Message::Submit);

        let content = column![
            container(conversation_view).height(Length::Fill),
            input_view,
            submit_button,
        ]
        .spacing(10)
        .padding(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
