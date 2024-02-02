use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;

use crate::{
    ai_functions::ai_func_architect::{print_project_scope, print_site_urls},
    helpers::{
        general::{ai_task_request_decoded, check_status_code},
        PrintCommand,
    },
    models::{
        basic_agent::{AgentState, BasicAgent},
        basic_trait::BasicTraits,
        FactSheet,
    },
};

use super::agent_traits::{ProjectScope, SpecialFunctions};

#[derive(Debug)]
pub struct AgentSolutionArchitect {
    attributes: BasicAgent,
}

impl AgentSolutionArchitect {
    pub fn new() -> Self {
        Self {
            attributes: BasicAgent {
                objective: String::from(
                    "Gather information and design solutions for website development",
                ),
                position: String::from("Solution Architect"),
                state: crate::models::basic_agent::AgentState::Discovery,
                memory: vec![],
            },
        }
    }

    async fn call_project_scope(&mut self, fact_sheet: &mut FactSheet) -> ProjectScope {
        let msg_context: String = format!("{}", fact_sheet.project_description);

        let ai_response = ai_task_request_decoded::<ProjectScope>(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_project_scope),
            print_project_scope,
        )
        .await;

        fact_sheet.project_scope = Some(ai_response.clone());

        self.attributes
            .update_state(crate::models::basic_agent::AgentState::Finished);

        ai_response
    }

    async fn call_determine_external_urls(
        &mut self,
        fact_sheet: &mut FactSheet,
        msg_context: String,
    ) {
        let ai_response = ai_task_request_decoded::<Vec<String>>(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_site_urls),
            print_site_urls,
        )
        .await;
        fact_sheet.external_urls = Some(ai_response);
        self.attributes.update_state(AgentState::UnitTesting);
    }
}

#[async_trait]
impl SpecialFunctions for AgentSolutionArchitect {
    fn get_attributes_from_agent(&self) -> &BasicAgent {
        &self.attributes
    }

    async fn execute(
        &mut self,
        fact_sheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while !matches!(self.attributes.state, AgentState::Finished) {
            match self.attributes.state {
                AgentState::Discovery => {
                    let project_scope = self.call_project_scope(fact_sheet).await;
                    if project_scope.is_external_urls_required {
                        self.call_determine_external_urls(
                            fact_sheet,
                            fact_sheet.project_description.clone(),
                        )
                        .await;
                        self.attributes.state = AgentState::UnitTesting;
                    }
                }
                AgentState::UnitTesting => {
                    let mut exclude_urls: Vec<String> = vec![];
                    let client: Client = Client::builder()
                        .timeout(Duration::from_secs(5))
                        .build()
                        .unwrap();
                    let urls: &Vec<String> = fact_sheet
                        .external_urls
                        .as_ref()
                        .expect("No URL object on factsheet");
                    for url in urls {
                        let endpoint_str = format!("Testing URL Endpoint: {}", url);
                        PrintCommand::UnitTest
                            .print_agent_message(&self.attributes.position, &endpoint_str);
                        match check_status_code(&client, url).await {
                            Ok(status_code) => {
                                if status_code != 200 {
                                    exclude_urls.push(url.clone());
                                }
                            }
                            Err(e) => println!("Error checking {}: {}", url, e),
                        }
                    }

                    if exclude_urls.len() > 0 {
                        let new_urls = fact_sheet
                            .external_urls
                            .as_ref()
                            .unwrap()
                            .iter()
                            .filter(|url| !exclude_urls.contains(url))
                            .cloned()
                            .collect();
                        fact_sheet.external_urls = Some(new_urls);
                    }

                    self.attributes.state = AgentState::Finished;
                }
                _ => {
                    self.attributes.state = AgentState::Finished;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_solution_architect() {
        let mut agent_solution_architect = AgentSolutionArchitect::new();

        let mut fact_sheet = FactSheet {
            project_description: String::from("Build a fullstack website with user login and logout that shows latest Forex prices"),
            project_scope: None,
            external_urls: None,
            backend_code: None,
            api_endpoint_schema: None,

        };

        agent_solution_architect
            .execute(&mut fact_sheet)
            .await
            .expect("Unable to execute solution architect agent");

        assert!(fact_sheet.project_scope != None);
        assert!(fact_sheet.external_urls != None);

        dbg!(fact_sheet);
    }
}
