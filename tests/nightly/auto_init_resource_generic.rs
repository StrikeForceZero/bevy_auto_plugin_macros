use bevy_app::prelude::*;
use bevy_auto_plugin_macros::auto_plugin::*;
use bevy_ecs::prelude::*;

#[auto_init_resource(Test<bool>)]
#[derive(Resource, Default)]
struct Test<T>(T);

#[auto_plugin(app=app)]
fn plugin(app: &mut App) {}

fn app() -> App {
    let mut app = internal_test_util::create_minimal_app();
    app.add_plugins(plugin);
    app
}

#[test]
fn test_auto_init_resource_generic() {
    let app = app();
    assert!(
        app.world().get_resource::<Test<bool>>().is_some(),
        "did not auto init resource"
    );
}
