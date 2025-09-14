use std::io::{self, Write};
use std::process::Command;
use std::env;
use llm::{Model, InferenceResponse, InferenceFeedback}; // Correctly import all necessary types

mod finetune;
mod gui;

/// Executes a shell command and returns the output.
fn execute_command(command_str: &str) {
    println!("[Executing Command: '{}']", command_str);

    let output = Command::new("sh")
        .arg("-c")
        .arg(command_str)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("[Output]");
                io::stdout().write_all(&output.stdout).unwrap();
            } else {
                println!("[Error]");
                io::stderr().write_all(&output.stderr).unwrap();
            }
        }
        Err(e) => {
            eprintln!("[Failed to execute command]: {}", e);
        }
    }
}

/// The main command-line interface loop for interacting with the AI.
fn run_cli() {
    // --- To enable the real AI, uncomment the following block ---
    /*
    let model_path = "models/dolphin-2.2.1-mistral-7b.Q5_K_M.gguf"; // Or your chosen model
    let model = match llm::load::<llm::models::Llama>(
        &std::path::Path::new(model_path),
        Default::default(),
        llm::load_progress_callback_stdout,
    ) {
        Ok(model) => model,
        Err(err) => {
            panic!("FATAL: Failed to load model from {}: {}", model_path, err);
        }
    };
    let mut session = model.start_session(Default::default());
    println!("AI Model Loaded. Starting CLI session.");
    */

    // If the block above is commented out, the assistant runs in mocked mode.
    println!("AI Assistant Initialized. (Running in mocked CLI mode)");
    println!("Type 'exit' or 'quit' to end the session.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).unwrap() == 0 {
            println!();
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            break;
        }

        // --- MOCKED RESPONSE (if real model is commented out) ---
        println!("\n[AI Response]");
        let ai_response = if input.eq_ignore_ascii_case("ls") {
            "!cmd:ls -l".to_string()
        } else {
            format!("If I were a real AI, I would have processed your request: '{}'", input)
        };
        println!("{}", ai_response);

        // --- REAL INFERENCE (Uncomment this block to use the model) ---
        /*
        println!("\n[AI Response]");
        let mut generated_response = String::new();
        let res = session.infer::<std::convert::Infallible>(
            &model,
            &mut rand::thread_rng(),
            &llm::InferenceRequest {
                prompt: input.into(),
                ..Default::default()
            },
            &mut Default::default(),
            |r| {
                match r {
                    InferenceResponse::InferredToken(t) => {
                        print!("{t}");
                        std::io::stdout().flush().unwrap();
                        generated_response.push_str(&t);
                        Ok(InferenceFeedback::Continue)
                    },
                    _ => Ok(InferenceFeedback::Continue)
                }
            },
        );
        if let Err(e) = res {
            println!("\nInference failed: {e}");
        }
        println!();
        let ai_response = generated_response;
        */

        println!("\n------------------------------------");

        const CMD_PREFIX: &str = "!cmd:";
        if ai_response.starts_with(CMD_PREFIX) {
            let command_to_run = ai_response[CMD_PREFIX.len()..].trim();
            execute_command(command_to_run);
            println!("\n------------------------------------");
        }
    }
}

/// The main entry point of the application.
fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--cli" => {
                run_cli();
                Ok(())
            },
            "--finetune" => {
                finetune::run_finetuning();
                Ok(())
            },
            _ => {
                println!("Unknown argument '{}'. Defaulting to GUI mode.", args[1]);
                gui::run_gui()
            }
        }
    } else {
        gui::run_gui()
    }
}
