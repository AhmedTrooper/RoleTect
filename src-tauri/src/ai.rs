use rig::providers::gemini::Client;
use rig::client::CompletionClient;
use rig::completion::Prompt;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct JobDetails {
    pub title: String,
    pub company: String,
    pub requirements: Vec<String>,
    pub core_responsibilities: Vec<String>,
}

pub async fn parse_job_description(api_key: &str, raw_jd: &str) -> Result<JobDetails, String> {
    // FIX: Unpack the Result. If Client::new fails, it safely returns the error as a String.
    let client = Client::new(api_key)
        .map_err(|e| format!("Failed to initialize AI client: {}", e))?;
        // println!("Password is : {}", api_key);

        
    
    // Now 'client' is safely unwrapped, and we can build the extractor
    let extractor = client.extractor::<JobDetails>("gemini-3-flash-preview").build();
        
    let result = extractor.extract(raw_jd).await.map_err(|e| format!("AI Parsing Error: {}", e))?;
    println!("Extracted Job Details: {:?}", &result);
    
    Ok(result)
}

pub async fn tailor_latex_for_job(
    api_key: &str,
    base_latex: &str,
    raw_job_content: &str,
    custom_instruction: Option<&str>,
) -> Result<String, String> {
    let client = Client::new(api_key)
        .map_err(|e| format!("Failed to initialize AI client: {}", e))?;
        // println!("Tailor function is called!");

    let system_prompt = r#"You are an expert resume tailoring AI. Your task is to take a base LaTeX resume template and tailor it to match a specific job description. 
    
Rules:
1. Only modify the resume content, NOT the structure or LaTeX commands
2. Highlight keywords and experiences that match the job description
3. Keep all original sections and formatting
4. Output ONLY valid LaTeX code with no markdown, no explanations, no code fences
5. Ensure the output is a valid, compilable LaTeX document


If custom instructions are provided, prioritize them."#;

    let user_prompt = format!(
        r#"Base LaTeX Resume:
{}

Job Description:
{}

{}

Please tailor the resume to match the job description. Return only the modified LaTeX code."#,
        base_latex,
        raw_job_content,
        custom_instruction
            .map(|ci| format!("Custom Instructions:\n{}", ci))
            .unwrap_or_default()
    );

    let agent = client
        .agent("gemini-3-flash-preview")
        .preamble(system_prompt)
        .build();

    let response = agent
        .prompt(&user_prompt)
        .await
        .map_err(|e| format!("AI Tailoring Error: {}", e))?;

        println!("\n\n\nTailored LaTeX Resume:\n{}\n\n\n----->", &response);

    Ok(response)
}