use hello_agents::{App, Model, Result};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let app = App::from_model(Model::Compatible("gpt-5.5".to_owned()));
    let mut history = app.new_chat_history();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    println!("连续对话已启动，输入 exit / quit / :q 退出。");

    loop {
        print!("\nUser> ");
        stdout.flush()?;

        let mut prompt = String::new();
        if stdin.read_line(&mut prompt)? == 0 {
            break;
        }

        let prompt = prompt.trim();
        if prompt.is_empty() {
            continue;
        }
        if matches!(prompt, "exit" | "quit" | ":q") {
            break;
        }

        print!("Assistant> ");
        stdout.flush()?;
        app.stream_chat_to_stdout(prompt, &mut history).await?;
    }

    Ok(())
}
