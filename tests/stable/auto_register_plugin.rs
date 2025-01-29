use bevy_app::prelude::*;
use bevy_auto_plugin::auto_plugin_module::*;
use bevy_ecs::prelude::*;
use bevy_reflect::prelude::*;
use std::any::Any;
use bevy_registration::RegistrationPlugin;

#[auto_plugin(init_name=init)]
mod plugin_module {
    use super::*;

    #[auto_register_type]
    #[derive(Reflect)]
    pub struct Test;
}
use plugin_module::*;


fn app() -> App {
    let mut app = internal_test_util::create_minimal_app();
    app.add_plugins(RegistrationPlugin);
    app
}

#[test]
fn test_auto_register_type() {
    let app = app();
    let type_registry = app.world().resource::<AppTypeRegistry>().0.clone();
    let type_registry = type_registry.read();
    assert!(
        type_registry.contains(Test.type_id()),
        "did not auto register type"
    );
}
