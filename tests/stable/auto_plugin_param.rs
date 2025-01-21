use bevy_app::prelude::*;
use bevy_auto_plugin_macros::auto_plugin_module::*;
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use std::any::Any;

#[auto_plugin(init_name=init)]
mod plugin_module {
    use super::*;

    #[auto_register_type]
    #[derive(Reflect)]
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
fn test_auto_plugin_param() {
    let app = app();
    let type_registry = app.world().resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();
    assert!(
        type_registry.contains(Test.type_id()),
        "did not auto register type"
    );
}
