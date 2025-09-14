use std::io::{self, Write};
use std::process::Command;
use std::env;

mod finetune;
mod gui;

/// Executes a shell command and returns the output.
/// This function is currently only used by the CLI mode.
/// A similar logic would be needed for the GUI to handle commands.
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
    println!("AI Assistant Initialized. (Running in CLI mode)");
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

        println!("\n[AI Response]");
        let ai_response = if input.eq_ignore_ascii_case("ls") {
            "!cmd:ls -l".to_string()
        } else {
            format!("If I were a real AI, I would have processed your request: '{}'", input)
        };
        println!("{}", ai_response);
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
/// It parses command-line arguments to decide which mode to run.
fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--cli" => {
                run_cli();
                Ok(()) // Return Ok since this path doesn't use iced
            },
            "--finetune" => {
                finetune::run_finetuning();
                Ok(()) // Return Ok for this path as well
            },
            _ => {
                // Default to GUI if the argument is unknown or not specified
                println!("Unknown argument '{}'. Defaulting to GUI mode.", args[1]);
                gui::run_gui()
            }
        }
    } else {
        // No arguments, run the GUI by default
        gui::run_gui()
    }
}
