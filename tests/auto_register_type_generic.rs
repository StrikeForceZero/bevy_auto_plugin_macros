use bevy::prelude::*;
use bevy_auto_plugin_macros::*;
use std::any::Any;

#[auto_register_type(Test<bool>)]
#[derive(Reflect)]
struct Test<T>(T);

#[auto_plugin(app=app)]
fn plugin(app: &mut App) {}

fn app() -> App {
    let mut app = internal_test_util::create_minimal_app();
    app.add_plugins(plugin);
    app
}

#[test]
fn test_auto_register_type_generic() {
    let app = app();
    let type_registry = app.world().resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();
    assert!(
        type_registry.contains(Test(true).type_id()),
        "did not auto register type"
    );
}
