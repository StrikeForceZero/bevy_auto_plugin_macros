use bevy_app::prelude::*;
use bevy_auto_plugin::auto_plugin_module::*;
use bevy_ecs::prelude::*;

#[auto_plugin(init_name=init)]
mod plugin_module {
    use super::*;

    #[auto_init_resource]
    #[derive(Resource, Default)]
    pub struct Test;
}
use plugin_module::*;

fn plugin(app: &mut App) {
    plugin_module::init(app);
}

fn app() -> App {
    let mut app = internal_test_util::create_minimal_app();
    app.add_plugins(plugin);
    app
}

#[test]
fn test_auto_init_resource() {
    let app = app();
    assert!(
        app.world().get_resource::<Test>().is_some(),
        "did not auto init resource"
    );
}
