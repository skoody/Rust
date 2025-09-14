use anyhow::{Error as E, Result};
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::llama as model;
use hf_hub::{api::sync::Api, Repo, RepoType};
use std::io::Write;
use tokenizers::Tokenizer;

fn main() -> Result<()> {
    // --- 1. Setup ---
    // This will select the GPU if available, otherwise CPU.
    // You can also change `Device::Cpu` to `Device::new_cuda(0)?` or `Device::new_metal(0)?`
    let device = Device::cuda_if_available(0)?;
    println!("Using device: {:?}", device);

    // --- 2. Load Model and Tokenizer ---
    // This section downloads the model and tokenizer from the Hugging Face Hub.
    // It will be cached locally for future runs.
    // IMPORTANT: Replace the model_id with the one you want to use.
    //            "TinyLlama/TinyLlama-1.1B-Chat-v1.0" is a good starting point.
    let api = Api::new()?;
    let repo = api.repo(Repo::new(
        "TinyLlama/TinyLlama-1.1B-Chat-v1.0".to_string(),
        RepoType::Model,
    ));

    println!("Loading tokenizer...");
    let tokenizer_filename = repo.get("tokenizer.json")?;
    let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

    println!("Loading model weights...");
    let config_filename = repo.get("config.json")?;
    let llama_config: model::LlamaConfig =
        serde_json::from_slice(&std::fs::read(config_filename)?)?;
    let config = llama_config.into_config(false); // Convert to the internal Config

    let model_filename = repo.get("model.safetensors")?;
    let vb = unsafe {
        candle_nn::VarBuilder::from_mmaped_safetensors(
            &[model_filename],
            candle_core::DType::F16,
            &device,
        )?
    };
    let mut model = model::Llama::load(vb, &config)?;
    let mut cache = model::Cache::new(true, candle_core::DType::F16, &config, &device)?;
    println!("Model loaded successfully.");

    // --- 3. Inference Loop ---
    let mut logits_processor = LogitsProcessor::new(299792458, Some(0.8), None);
    let mut tokens = vec![];

    loop {
        print!("\n> ");
        std::io::stdout().flush()?;
        let mut prompt = String::new();
        std::io::stdin().read_line(&mut prompt)?;

        if prompt.trim().eq_ignore_ascii_case("exit") || prompt.trim().eq_ignore_ascii_case("quit") {
            break;
        }

        let prompt_tokens = tokenizer
            .encode(prompt.as_str(), true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();

        tokens.extend_from_slice(&prompt_tokens);

        println!("\n[AI Response]");
        let mut generated_tokens = 0;
        let eos_token = *tokenizer.get_vocab(true).get("</s>").unwrap();

        for index in 0..1000 { // Generate up to 1000 tokens
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let input = Tensor::new(&tokens[start_pos..], &device)?.unsqueeze(0)?;
            let logits = model.forward(&input, start_pos, &mut cache)?;
            let logits = logits.squeeze(0)?.to_dtype(candle_core::DType::F32)?;

            let next_token = logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens += 1;

            if next_token == eos_token {
                break;
            }

            let token_str = tokenizer.decode(&[next_token], true).map_err(E::msg)?;
            print!("{}", token_str);
            std::io::stdout().flush()?;
        }
    }

    Ok(())
}
