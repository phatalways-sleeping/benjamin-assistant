use crate::models::{APIResponse, ChatCompletion, Message};
use dotenv::dotenv;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use std::env;

// Call LLM
pub async fn call_gpt(messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();

    let api_key = env::var("OPEN_AI_KEY").expect("Open AI Key not found");
    let org_key = env::var("OPEN_AI_ORG").expect("Open AI Org Key not found");
    let model = env::var("GPT_MODEL").expect("LLM name not found");
    let url = "https://api.openai.com/v1/chat/completions";

    // Headers
    let mut headers = HeaderMap::new();

    // Open AI Header
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|err| -> Box<dyn std::error::Error + Send> { Box::new(err) })?,
    );

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );

    // Open AI Org Header
    headers.insert(
        "OpenAI-Organization",
        HeaderValue::from_str(&org_key)
            .map_err(|err| -> Box<dyn std::error::Error + Send> { Box::new(err) })?,
    );

    // Create client
    let client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|err| -> Box<dyn std::error::Error + Send> { Box::new(err) })?;

    // Create chat completion
    let chat_completion = ChatCompletion {
        model,
        messages,
        temperature: 0.1,
    };

    // Response
    let res = client
        .post(url)
        .json(&chat_completion)
        .send()
        .await
        .map_err(|err| -> Box<dyn std::error::Error + Send> { Box::new(err) })?;

    let res = res.json::<APIResponse>()
    .await
    .map_err(|err| -> Box<dyn std::error::Error + Send> { Box::new(err) })?;

    Ok(res.choices[0].message.content.clone())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_call_gpt() {
        let message = Message {
            role: String::from("user"),
            content: String::from("HI there, this is a test. Give me a short response"),
        };

        let messages = vec![message];

        let response = call_gpt(messages).await;

        if let Ok(res_str) = response {
            dbg!(res_str);
        } else {
            panic!()
        }
    }
}
