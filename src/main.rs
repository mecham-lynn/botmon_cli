use botmon_cli::{app::AppState, error::install_hooks, init, restore, AppParams};

#[tokio::main]
async fn main() -> color_eyre::Result<()>{
    let args: AppParams = argh::from_env();
    install_hooks()?;
    let mut terminal = init()?;
    let mut app = AppState::new(&args).await?;
    let app_result = app.run(&mut terminal).await;
    restore()?;
    app_result
}
