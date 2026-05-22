use hello_agents::{App, Model};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let app = App::from_model(Model::Compatible("gpt-5.5".to_owned()));
    let _result = app
        .stream_to_stdout("hello, how is the weather today in beijing")
        .await
        .unwrap();
}
