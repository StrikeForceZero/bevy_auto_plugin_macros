use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_auto_plugin_macros::*;

#[auto_add_event(Test<bool>)]
#[derive(Event, Debug, PartialEq)]
struct Test<T>(T);

#[auto_plugin(app=app)]
fn plugin(app: &mut App) {}

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
