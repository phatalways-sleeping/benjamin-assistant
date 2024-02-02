use std::{env, error::Error, fs};

use dotenv::dotenv;
use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::{apis::call_gpt, models::Message};

use super::PrintCommand;

pub fn extend_ai_function(ai_func: fn(&str) -> &'static str, func_input: &str) -> Message {
    let ai_function_str = ai_func(func_input);

    let msg: String = format!(
        "FUNCTION: {}
    INSTRUCTION: You are a function printer. You ONLY print the results of functions.
    Nothing else. No commentary. Here is the input to the function: {}.
    Print out what the function will return.",
        ai_function_str, func_input
    );

    Message {
        role: "system".to_string(),
        content: msg,
    }
}

// Perform calls to LLM
pub async fn ai_task_request(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_passed: for<'a> fn(&'a str) -> &'static str,
) -> String {
    let func_msg = extend_ai_function(function_passed, &msg_context);

    PrintCommand::AICall.print_agent_message(agent_position, agent_operation);

    let llm_response_res: Result<String, Box<dyn Error + Send>> =
        call_gpt(vec![func_msg.clone()]).await;

    let llm_response = match llm_response_res {
        Ok(llm_resp) => llm_resp,
        Err(_) => call_gpt(vec![func_msg.clone()])
            .await
            .expect("Failed twice to open OpenAI"),
    };

    llm_response
}

// Decode into a certain struct
pub async fn ai_task_request_decoded<T: DeserializeOwned>(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_passed: for<'a> fn(&'a str) -> &'static str,
) -> T {
    let llm_response = ai_task_request(
        msg_context,
        agent_position,
        agent_operation,
        function_passed,
    )
    .await;

    let decoded_response: T = serde_json::from_str(llm_response.as_str())
        .expect("Failed to decode ai response from serde_json");

    decoded_response
}

// Check whether request url is valid
pub async fn check_status_code(client: &Client, url: &str) -> Result<u16, reqwest::Error> {
    let response = client.get(url).send().await?;
    Ok(response.status().as_u16())
}

// Get Code Template
pub fn read_code_template_contents() -> String {
    dotenv().ok();
    let code_template_absolute_path =
        env::var("CODE_TEMPLATE_ABSOLUTE_PATH").expect("CODE_TEMPLATE_ABSOLUTE_PATH Key not found");
    let path: String = String::from(code_template_absolute_path);
    fs::read_to_string(path).expect("Failed to read code template")
}

// Get Main Template
pub fn read_exec_main_contents() -> String {
    dotenv().ok();
    let execute_main_absolute_path =
        env::var("EXEC_MAIN_ABSOLUTE_PATH").expect("EXEC_MAIN_ABSOLUTE_PATH Key not found");
    let path: String = String::from(execute_main_absolute_path);
    fs::read_to_string(path).expect("Failed to read code template")
}

// Save new backend codes
pub fn save_backend_code(contents: &str) {
    dotenv().ok();
    let execute_main_absolute_path =
        env::var("EXEC_MAIN_ABSOLUTE_PATH").expect("EXEC_MAIN_ABSOLUTE_PATH Key not found");
    let path: String = String::from(execute_main_absolute_path);
    fs::write(path, contents).expect("Failed to write main.rs file")
}

// Save JSON API Endpoint Schema
pub fn save_api_endpoints(api_endpoints: &str) {
    dotenv().ok();
    let api_schema_absolute_path =
        env::var("API_SCHEMA_ABSOLUTE_PATH").expect("API_SCHEMA_ABSOLUTE_PATH Key not found");
    let path: String = String::from(api_schema_absolute_path);
    fs::write(path, api_endpoints).expect("Failed to write api endpoints to file")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ai_functions::{
        ai_func_architect::print_project_scope, ai_func_managing::convert_user_input_to_goal,
    };

    #[test]
    fn test_extend_ai_function() {
        let msg = extend_ai_function(print_project_scope, "dummy variable");

        assert_eq!(msg.role, "system".to_string());
    }

    #[tokio::test]
    async fn test_ai_task_request() {
        let response = ai_task_request(
            "Build me a website for making stock price API requests".to_string(),
            "Managing Agent",
            "Defining user requirements",
            convert_user_input_to_goal,
        )
        .await;

        assert!(response.len() > 30);
    }
}
