# Bevy Auto Plugin Macros

##

## Usage - Stable
```rust
use bevy::prelude::*;
use bevy_auto_plugin_macros::auto_plugin_module::*;

#[auto_plugin(init_name=init)]
mod plugin_module {
    use super::*;
    
    #[auto_register_type]
    #[derive(Component, Reflect)]
    #[reflect(Component)]
    pub struct FooComponent;

    #[auto_register_type(FooComponentWithGeneric<bool>)]
    #[auto_register_type(FooComponentWithGeneric<u32>)]
    #[derive(Component, Reflect)]
    #[reflect(Component)]
    pub struct FooComponentWithGeneric<T>(T);

    #[auto_register_type]
    #[auto_add_event]
    #[derive(Event, Reflect)]
    pub struct FooEvent;

    #[auto_register_type(FooEvent<bool>)]
    #[auto_add_event]
    #[derive(Event, Reflect)]
    pub struct FooEventWithGeneric<T>(T);

    #[auto_register_type]
    #[auto_init_resource]
    #[derive(Resource, Default, Reflect)]
    #[reflect(Resource)]
    pub struct FooResource;

    #[auto_register_type(FooResourceWithGeneric<bool>)]
    #[auto_init_resource]
    #[derive(Resource, Default, Reflect)]
    #[reflect(Resource)]
    pub struct FooResourceWithGeneric<T>(T);
}

#[auto_plugin(app=app)]
fn plugin(app: &mut App) {
    plugin_module::init(app);
}
```

Which generates this code
```rust
mod plugin_module {
    // ...
    fn ini(app: &mut App) {
        app.register_type::<FooComponent>();
        app.register_type::<FooComponentWithGeneric<bool>>();
        app.register_type::<FooComponentWithGeneric<u32>>();
        app.register_type::<FooEvent>();
        app.register_type::<FooEventWithGeneric<bool>>();
        app.register_type::<FooResource>();
        app.register_type::<FooResourceWithGeneric<bool>>();

        app.add_event::<FooEvent>();
        app.add_event::<FooEventWithGeneric<bool>>();

        app.init_resource::<FooResource>();
        app.init_resource::<FooResourceWithGeneric<bool>>();
    }
}
```

### Known Limitations
- Causes issues for ide's like RustRover

## Usage - [Nightly](https://github.com/StrikeForceZero/bevy_auto_plugin_macros/tree/nightly)
```rust
use bevy::prelude::*;
use bevy_auto_plugin_macros::*;

#[auto_register_type]
#[derive(Component, Reflect)]
#[reflect(Component)]
struct FooComponent;

#[auto_register_type(FooComponentWithGeneric<bool>)]
#[auto_register_type(FooComponentWithGeneric<u32>)]
#[derive(Component, Reflect)]
#[reflect(Component)]
struct FooComponentWithGeneric<T>(T);

#[auto_register_type]
#[auto_add_event]
#[derive(Event, Reflect)]
struct FooEvent;

#[auto_register_type(FooEvent<bool>)]
#[auto_add_event]
#[derive(Event, Reflect)]
struct FooEventWithGeneric<T>(T);

#[auto_register_type]
#[auto_init_resource]
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct FooResource;

#[auto_register_type(FooResourceWithGeneric<bool>)]
#[auto_init_resource]
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
struct FooResourceWithGeneric<T>(T);

#[auto_plugin(app=app)]
fn plugin(app: &mut App) {}
```

Which generates this code in your fn accepting `&mut App`
```rust
#[auto_plugin(app=app)]
fn plugin(app: &mut App) {
    app.register_type::<FooComponent>();
    app.register_type::<FooComponentWithGeneric<bool>>();
    app.register_type::<FooComponentWithGeneric<u32>>();
    app.register_type::<FooEvent>();
    app.register_type::<FooEventWithGeneric<bool>>();
    app.register_type::<FooResource>();
    app.register_type::<FooResourceWithGeneric<bool>>();
    
    app.add_event::<FooEvent>();
    app.add_event::<FooEventWithGeneric<bool>>();
    
    app.init_resource::<FooResource>();
    app.init_resource::<FooResourceWithGeneric<bool>>();
    
    // ...
}
```

### Known Limitations
- The internal state relies on call site file paths which currently requires `Nightly` rust.

- All items need to be in the same module. This won't work:
```rust
use bevy::prelude::*;
use bevy_auto_plugin_macros::*;
mod foo {
    use super::*;
    #[auto_register_type]
    #[derive(Component, Reflect)]
    #[reflect(Component)]
    struct FooComponent;
}

#[auto_plugin(app=app)]
fn plugin(app: &mut App) {
    // ...
}
```

## License

All code in this repository is dual-licensed under either:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option. This means you can select the license you prefer.

### Your Contributions

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual-licensed as above, without any additional terms or conditions.