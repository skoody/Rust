use anyhow::Result;
use llama_cpp::{LlamaModel, LlamaParams, SessionParams, StandardSampler};
use std::io::{self, Write};

fn main() -> Result<()> {
    // --- 1. Load the Model ---
    // IMPORTANT: Replace this with the actual path to your GGUF model file.
    // For example: "/home/skoody/Documents/Models/dolphin-2.2.1-mistral-7b.Q5_K_M.gguf"
    let model_path = "models/model.gguf"; // A placeholder path
    println!("Attempting to load model from: {}", model_path);
    println!("Please ensure you have a model file at this location or update the path.");

    let model = match LlamaModel::load_from_file(model_path, LlamaParams::default()) {
        Ok(model) => model,
        Err(e) => {
            anyhow::bail!("Failed to load model from '{}'. Error: {}. Please check the path and ensure the model file is valid.", model_path, e);
        }
    };
    println!("Model loaded successfully.");

    // --- 2. Create a Session ---
    // A session holds the context for a conversation.
    let mut ctx = model.create_session(SessionParams::default())?;

    println!("\n--- AI Assistant Ready ---");
    println!("Enter a prompt or type 'exit' to quit.");

    // --- 3. Interaction Loop ---
    loop {
        print!("\n> ");
        io::stdout().flush()?;
        let mut prompt = String::new();
        io::stdin().read_line(&mut prompt)?;

        let prompt_trimmed = prompt.trim();
        if prompt_trimmed.is_empty() {
            continue;
        }
        if prompt_trimmed.eq_ignore_ascii_case("exit") || prompt_trimmed.eq_ignore_ascii_case("quit") {
            break;
        }

        // Add the user's prompt to the context.
        // The `\n` is important for some models to recognize the end of a prompt.
        ctx.advance_context(prompt_trimmed)?;
        ctx.advance_context("\n")?;

        // Generate the AI's response
        let mut completions =
            ctx.start_completing_with(StandardSampler::default(), 1024);

        print!("[AI Response]: ");
        io::stdout().flush()?;

        // Stream the tokens as they are generated.
        for completion in completions.into_strings() {
            print!("{}", completion);
            let _ = io::stdout().flush();
        }
    }

    Ok(())
}
