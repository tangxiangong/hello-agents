use hello_agents::{App, Model};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = App::from_model(Model::Compatible("gpt-5.5".to_owned()));
    let prompt = "你好，请帮我查询一下今天北京的天气，然后根据天气推荐一个合适的旅游景点。";
    println!("User: {}", prompt);
    println!("Assistant: ");
    let _result = app.stream_to_stdout(prompt).await.unwrap();
}
