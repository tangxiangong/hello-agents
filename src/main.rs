use hello_agents::{App, Model};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = App::from_model(Model::OpenAI(hello_agents::OpenAIModel::GPT_5_5));
    let result = app.run("hello").await.unwrap();
    println!("result: {}", result);
}
