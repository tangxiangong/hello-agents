use hello_agents::{App, Model};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = App::from_model(Model::Compatible("gpt-5.5".to_owned()));
    let result = app
        .run("hello, how is the weather today in beijing")
        .await
        .unwrap();
    println!("result: {}", result);
}
