use bevy_auto_plugin::auto_plugin::*;


#[auto_plugin(app=_app)]
fn plugin(_app: &mut bevy_app::App) {}

#[auto_register_type]
struct Test;

// dummy main
fn main() {
    
}