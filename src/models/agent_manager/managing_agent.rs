use crate::{
    ai_functions::ai_func_managing::convert_user_input_to_goal,
    helpers::general::ai_task_request,
    models::{
        agent_architect::AgentSolutionArchitect,
        agent_backend::AgentBackendDeveloper,
        agents::agent_traits::SpecialFunctions,
        basic_agent::{AgentState, BasicAgent},
        FactSheet,
    },
};

#[derive(Debug)]
pub struct ManagingAgent {
    _attributes: BasicAgent,
    fact_sheet: FactSheet,
    agents: Vec<Box<dyn SpecialFunctions>>,
}

impl ManagingAgent {
    pub async fn new(user_req: String) -> Result<Self, Box<dyn std::error::Error>> {
        let attributes = BasicAgent {
            objective: "Manage agents who are building an excellent website for the user"
                .to_string(),
            position: "Project Manager".to_string(),
            state: AgentState::Discovery,
            memory: vec![],
        };

        let project_description = ai_task_request(
            user_req,
            &attributes.position,
            get_function_string!(convert_user_input_to_goal),
            convert_user_input_to_goal,
        )
        .await;

        let agents: Vec<Box<dyn SpecialFunctions>> = vec![];

        let fact_sheet = FactSheet {
            project_description,
            project_scope: None,
            external_urls: None,
            backend_code: None,
            api_endpoint_schema: None,
        };

        Ok(Self {
            _attributes: attributes,
            fact_sheet,
            agents,
        })
    }

    fn add_agent(&mut self, agent: Box<dyn SpecialFunctions>) {
        self.agents.push(agent);
    }

    fn create_agents(&mut self) {
        self.add_agent(Box::new(AgentSolutionArchitect::new()));
        self.add_agent(Box::new(AgentBackendDeveloper::new()));
    }

    pub async fn execute_project(&mut self) {
        self.create_agents();

        for agent in self.agents.iter_mut() {
            let _: Result<(), Box<dyn std::error::Error>> =
                agent.execute(&mut self.fact_sheet).await;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_managing_agent() {
        let user_request = "need a full stack that fetch and tracks my fitness progress. Needs to include timezone info from the web.";

        let mut managing_agent = ManagingAgent::new(user_request.to_string())
            .await
            .expect("Error creating managing agent");

        managing_agent.execute_project().await;
    }
}
