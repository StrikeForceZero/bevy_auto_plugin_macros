use bevy_auto_plugin::auto_plugin::*;


#[auto_register_type(Test)]
struct Test<T>(T);

#[auto_plugin(app=_app)]
fn plugin(_app: &mut bevy_app::App) {}

// dummy main
fn main() {
    
}