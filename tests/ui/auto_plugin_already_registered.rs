use bevy::prelude::*;
use bevy_auto_plugin_macros::*;


#[auto_plugin(app=_app)]
fn plugin(_app: &mut App) {}

#[auto_register_type]
struct Test;

// dummy main
fn main() {
    
}