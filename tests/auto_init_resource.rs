use bevy::prelude::*;
use bevy_auto_plugin_macros::*;

#[auto_init_resource]
#[derive(Resource, Default)]
struct Test;

#[auto_plugin(app=app)]
fn plugin(app: &mut App) {

}

fn app() -> App {
    let mut app = internal_test_util::create_minimal_app();
    app.add_plugins(plugin);
    app
}

#[test]
fn test_auto_init_resource() {
    let app = app();
    assert!(app.world().get_resource::<Test>().is_some(), "did not auto init resource");
}