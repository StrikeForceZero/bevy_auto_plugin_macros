use bevy::prelude::*;

pub fn create_minimal_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}
