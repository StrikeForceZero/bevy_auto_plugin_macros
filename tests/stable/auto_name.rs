use bevy_app::prelude::*;
use bevy_auto_plugin_macros::auto_plugin_module::*;
use bevy_ecs::prelude::*;
use bevy_core::Name;

#[auto_plugin(init_name=init)]
mod plugin_module {
    use super::*;
    #[derive(Component)]
    #[auto_name]
    pub struct Test;
}
use plugin_module::*;

fn app() -> App {
    let mut app = internal_test_util::create_minimal_app();
    app.add_plugins(init);
    app
}

#[test]
fn test_auto_name() {
    let mut app = app();
    let entity = app.world_mut().spawn(Test).id();
    app.update();
    assert_eq!(app.world().entity(entity).get::<Name>(), Some(&Name::new("Test")));
}
