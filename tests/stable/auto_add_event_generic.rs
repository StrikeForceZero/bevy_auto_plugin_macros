use bevy_app::prelude::*;
use bevy_auto_plugin_macros::auto_plugin_module::*;
use bevy_ecs::prelude::*;

#[auto_plugin(init_name=init)]
mod plugin_module {
    use super::*;
    #[auto_add_event(Test<bool>)]
    #[derive(Event, Debug, PartialEq)]
    pub struct Test<T>(pub T);
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
fn test_auto_add_event_generic() {
    let mut app = app();
    let mut events = app.world_mut().resource_mut::<Events<Test<bool>>>();
    events.send(Test(true));
    assert_eq!(
        events.drain().next(),
        Some(Test(true)),
        "did not auto add event"
    );
    assert_eq!(events.drain().next(), None, "expected only 1 event");
}
