use std::{
    env, process::{Command, Stdio}, time::Duration
};

use async_trait::async_trait;
use dotenv::dotenv;
use reqwest::Client;
use tokio::time::sleep;

use crate::{
    ai_functions::ai_func_backend::{
        print_backend_webserver_code, print_fixed_code, print_improved_webserver_code,
        print_rest_api_endpoints,
    },
    helpers::{
        confirm_safe_code,
        general::{
            ai_task_request,
            check_status_code,
            read_code_template_contents,
            read_exec_main_contents,
            save_api_endpoints,
            save_backend_code,
            // WEB_SERVER_PROJECT_PATH,
        },
        PrintCommand,
    },
    models::{
        basic_agent::{AgentState, BasicAgent},
        FactSheet,
    },
};

use super::agent_traits::{RouteObject, SpecialFunctions};

#[derive(Debug)]
pub struct AgentBackendDeveloper {
    attributes: BasicAgent,
    bug_errors: Option<String>,
    bug_count: u8,
}

impl AgentBackendDeveloper {
    pub fn new() -> Self {
        Self {
            attributes: BasicAgent {
                position: "Backend Developer".to_string(),
                objective: "Develop backend code for web server and json database".to_string(),
                state: AgentState::Discovery,
                memory: vec![],
            },
            bug_errors: None,
            bug_count: 0,
        }
    }

    async fn call_initial_backend_code(&mut self, fact_sheet: &mut FactSheet) {
        let code_template_str = read_code_template_contents();

        let msg_context = format!(
            "CODE TEMPLATE: {} \n PROJECT_DESCRIPTION: {} \n",
            code_template_str, fact_sheet.project_description
        );

        let ai_response = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_backend_webserver_code),
            print_backend_webserver_code,
        )
        .await;

        save_backend_code(&ai_response);
        fact_sheet.backend_code = Some(ai_response);
    }

    async fn call_improve_backend_code(&mut self, fact_sheet: &mut FactSheet) {
        let msg_context = format!(
            "CODE TEMPLATE: {:?} \n PROJECT_DESCRIPTION: {:?} \n",
            fact_sheet.backend_code, fact_sheet
        );

        let ai_response = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_improved_webserver_code),
            print_improved_webserver_code,
        )
        .await;

        save_backend_code(&ai_response);
        fact_sheet.backend_code = Some(ai_response);
    }

    async fn call_fix_code_bugs(&mut self, fact_sheet: &mut FactSheet) {
        let msg_context = format!(
            "BROKEN_CODE: {:?} \n ERROR_BUGS: {:?} \n THIS FUNCTION ONLY OUTPUTS CODE. JUST OUTPUT THE CODE.",
            fact_sheet.backend_code, self.bug_errors
        );

        let ai_response = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_fixed_code),
            print_fixed_code,
        )
        .await;

        save_backend_code(&ai_response);
        fact_sheet.backend_code = Some(ai_response);
    }

    async fn call_extract_rest_api_endpoints(&self) -> String {
        let backend_code = read_exec_main_contents();

        let msg_context = format!("CODE_INPUT: {:?}", backend_code);

        let ai_response = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_rest_api_endpoints),
            print_rest_api_endpoints,
        )
        .await;

        ai_response
    }
}

#[async_trait]
impl SpecialFunctions for AgentBackendDeveloper {
    fn get_attributes_from_agent(&self) -> &BasicAgent {
        &self.attributes
    }

    async fn execute(
        &mut self,
        fact_sheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while self.attributes.state != AgentState::Finished {
            match &self.attributes.state {
                AgentState::Discovery => {
                    self.call_initial_backend_code(fact_sheet).await;
                    self.attributes.state = AgentState::Working;
                    continue;
                }
                AgentState::Working => {
                    if self.bug_count == 0 {
                        self.call_improve_backend_code(fact_sheet).await;
                    } else {
                        self.call_fix_code_bugs(fact_sheet).await;
                    }
                    self.attributes.state = AgentState::UnitTesting;
                    continue;
                }
                AgentState::UnitTesting => {
                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position,
                        "Backend code unit testing: Requesting user input",
                    );
                    let is_safe_code = confirm_safe_code();
                    if !is_safe_code {
                        println!("Better go work on some AI alignment instead...");
                    }
                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position,
                        "Backend code unit testing: Building project...",
                    );
                    dotenv().ok();
                    let web_server_project_absolute_path = env::var("WEB_SERVER_PROJECT_ABSOLUTE_PATH")
                        .expect("WEB_SERVER_PROJECT_ABSOLUTE_PATH Key not found");
                    let build_backend_server = Command::new("cargo")
                        .arg("build")
                        .current_dir(web_server_project_absolute_path.clone())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .expect("Failed to build backend application");

                    if build_backend_server.status.success() {
                        self.bug_count = 0;
                        PrintCommand::UnitTest.print_agent_message(
                            &self.attributes.position,
                            "Backend code unit testing: Building project successfully",
                        );
                    } else {
                        let error_arr = build_backend_server.stderr;
                        let error_arr = String::from_utf8(error_arr).unwrap();

                        self.bug_count += 1;
                        self.bug_errors = Some(error_arr);

                        if self.bug_count > 2 {
                            PrintCommand::Issue.print_agent_message(
                                &self.attributes.position,
                                "Backend code unit testing: Too many bugs found in code",
                            );
                            panic!("Error: Too many bugs");
                        }

                        self.attributes.state = AgentState::Working;
                        continue;
                    }

                    let api_endpoints_str = self.call_extract_rest_api_endpoints().await;

                    let api_endpoints: Vec<RouteObject> = serde_json::from_str(&api_endpoints_str)
                        .expect("Failed to decode API endpoints");

                    let check_endpoints: Vec<RouteObject> = api_endpoints
                        .iter()
                        .filter(|&route| route.method == "get" && route.is_route_dynamic == "false")
                        .cloned()
                        .collect();

                    fact_sheet.api_endpoint_schema = Some(check_endpoints.clone());

                    // Run backend application
                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position,
                        "Backend code unit testing: Starting web server",
                    );

                    let mut run_backend_server = Command::new("cargo")
                        .arg("run")
                        .current_dir(web_server_project_absolute_path)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to run backend application");

                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position,
                        "Backend code unit testing: Lauching tests on server in 5 seconds...",
                    );

                    let seconds_sleep = Duration::from_secs(5);
                    sleep(seconds_sleep).await;

                    for endpoint in check_endpoints {
                        let testing_msg = format!("Testing endpoint '{}'...", endpoint.route);
                        PrintCommand::UnitTest
                            .print_agent_message(&self.attributes.position, &testing_msg);

                        let client = Client::builder()
                            .timeout(Duration::from_secs(5))
                            .build()
                            .unwrap();

                        let url = format!("http://localhost:8080{}", endpoint.route);
                        match check_status_code(&client, &url).await {
                            Ok(status_code) => {
                                if status_code != 200 {
                                    let err_msg = format!(
                                        "WARNING: Failed to call backend url endpoint {}",
                                        endpoint.route
                                    );
                                    PrintCommand::UnitTest
                                        .print_agent_message(&self.attributes.position, &err_msg);
                                }
                            }
                            Err(e) => {
                                run_backend_server
                                    .kill()
                                    .expect("Failed to kill backend server");
                                let err_msg = format!("Error checking backend {}", e);
                                PrintCommand::UnitTest
                                    .print_agent_message(&self.attributes.position, &err_msg);
                            }
                        }
                    }
                    save_api_endpoints(&api_endpoints_str);
                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position,
                        "Backend testing completed...",
                    );
                    run_backend_server
                        .kill()
                        .expect("Failed to kill backend server on completion");

                    self.attributes.state = AgentState::Finished;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
