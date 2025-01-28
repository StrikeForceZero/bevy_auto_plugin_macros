use bevy_app::prelude::*;
use bevy_auto_plugin::auto_plugin::*;
use bevy_ecs::prelude::*;

#[auto_add_event]
#[derive(Event)]
struct Test;

#[auto_plugin(app=app)]
fn plugin(app: &mut App) {}

fn app() -> App {
    let mut app = internal_test_util::create_minimal_app();
    app.add_plugins(plugin);
    app
}

#[test]
fn test_auto_add_event() {
    let mut app = app();
    let mut events = app.world_mut().resource_mut::<Events<Test>>();
    events.send(Test);
    assert_eq!(events.drain().count(), 1, "did not auto add event");
}
