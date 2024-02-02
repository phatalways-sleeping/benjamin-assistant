#[macro_export]
macro_rules! get_function_string {
    ($func: ident) => {{
        stringify!($func)
    }};
}

mod ai_functions;
mod apis;
mod helpers;
mod models;

use helpers::get_user_response;

use crate::models::ManagingAgent;
#[tokio::main]
async fn main() {
    let user_response = get_user_response("What website are we going to build today?");

    let mut managing_agent = ManagingAgent::new(user_response)
        .await
        .expect("Error creating agent");

    managing_agent.execute_project().await;
}
