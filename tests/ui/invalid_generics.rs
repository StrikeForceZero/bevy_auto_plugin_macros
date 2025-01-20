use bevy::prelude::*;
use bevy_auto_plugin_macros::*;


#[auto_register_type(Test)]
struct Test<T>(T);

#[auto_plugin(app=_app)]
fn plugin(_app: &mut App) {}

// dummy main
fn main() {
    
}