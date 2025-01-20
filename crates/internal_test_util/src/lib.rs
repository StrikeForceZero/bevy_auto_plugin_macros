use bevy_app::App;
use bevy_internal::MinimalPlugins;

pub fn create_minimal_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}
