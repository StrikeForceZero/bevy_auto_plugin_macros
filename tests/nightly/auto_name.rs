use bevy_app::prelude::*;
use bevy_auto_plugin_macros::auto_plugin::*;
use bevy_core::Name;
use bevy_ecs::prelude::*;

#[derive(Component)]
#[auto_name]
pub struct Test;

#[auto_plugin(app=app)]
fn plugin(app: &mut App) {}

fn app() -> App {
    let mut app = internal_test_util::create_minimal_app();
    app.add_plugins(plugin);
    app
}

#[test]
fn test_auto_name() {
    let mut app = app();
    let entity = app.world_mut().spawn(Test).id();
    app.update();
    assert_eq!(
        app.world().entity(entity).get::<Name>(),
        Some(&Name::new("Test"))
    );
}
