use std::io::{stdin, stdout};

use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};

#[derive(Debug, PartialEq)]
pub enum PrintCommand {
    AICall,
    UnitTest,
    Issue,
}

impl PrintCommand {
    pub fn print_agent_message(&self, agent_pos: &str, agent_statement: &str) {
        let mut stdout = stdout();

        let statement_color = match self {
            Self::AICall => Color::Cyan,
            Self::UnitTest => Color::Magenta,
            Self::Issue => Color::Red,
        };

        stdout.execute(SetForegroundColor(Color::Green)).unwrap();
        print!("Agent: {}: ", agent_pos);

        stdout.execute(SetForegroundColor(statement_color)).unwrap();
        println!("{}", agent_statement);

        stdout.execute(ResetColor).unwrap();
    }
}

pub fn get_user_response(question: &str) -> String {
    let mut stdout: std::io::Stdout = stdout();

    stdout.execute(SetForegroundColor(Color::Blue)).unwrap();

    println!("");
    println!("{}", question);

    stdout.execute(ResetColor).unwrap();

    // Read
    let mut user_response = String::new();

    stdin()
        .read_line(&mut user_response)
        .expect("Fail to read response");

    user_response.trim().to_string()
}

pub fn confirm_safe_code() -> bool {
    let mut stdout: std::io::Stdout = stdout();
    loop {
        stdout.execute(SetForegroundColor(Color::Blue)).unwrap();
        println!("");
        print!("WARNING: You are about to run code written entirely by AI.");
        println!("Review your code and confirm you wish to continue.");

        stdout.execute(ResetColor).unwrap();
        stdout.execute(SetForegroundColor(Color::Green)).unwrap();
        println!("[1] All good");
        stdout.execute(SetForegroundColor(Color::DarkRed)).unwrap();
        println!("[2] Let's stop this project");

        stdout.execute(ResetColor).unwrap();
        let mut response = String::new();
        stdin().read_line(&mut response).expect("Failed to read response");

        let response = response.trim().to_lowercase();

        match response.as_str() {
            "1" | "ok" | "y" => return true,
            "2" | "no"| "n" => return false,
            _ => {
                println!("Invalid input!");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::PrintCommand;

    #[test]
    fn test_print_agent_msg() {
        PrintCommand::AICall.print_agent_message("Managing agent", "Testing testing, process");
    }
}


