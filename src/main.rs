mod signup_flow;

use signup_flow::{create_account, SignupInput};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let mut args = std::env::args().skip(1);
    let email = args.next().unwrap_or_else(|| "owner@example.com".into());
    let tenant = args.next().unwrap_or_else(|| "demo".into());
    let input = SignupInput { email: email.clone(), password: "change-me-before-running".into(), name: "Workspace owner".into(), tenant };
    match create_account(input).await {
        Ok(message_id) => {
            println!("created account for {email}; verification message {message_id}");
            std::process::ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("signup request failed: {error:?}");
            std::process::ExitCode::FAILURE
        }
    }
}
