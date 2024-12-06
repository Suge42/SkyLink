use std::thread;
use crate::test::test_bench::{my_generic_fragment_forward, test_butterfly_flood, test_double_chain_flood, test_flood, test_star_flood, test_tree_flood};
use crate::initializer::initialize;

mod sim_app;
mod sim_control;
mod initializer;
mod skylink_drone;
mod test;

fn main() {
    println!("Hello, world!");
    //
    // let (sim_contr, handles) = initialize("input.toml");
    // sim_app::run_simulation_gui(sim_contr);
    //
    // for handle in handles.into_iter() {
    //     handle.join().unwrap();
    // }
    //
    // let test = false;
    // if test {
    //     //Comment functions we aren't testing
    //     //my_generic_fragment_forward();
    // } else {
    //
    // }

    // generic_chain_fragment_ack();
    // generic_fragment_drop();
    // generic_chain_fragment_drop();

    // test_generic();
   // test_double_chain_flood();
    //test_star_flood();
    // test_butterfly_flood()

    // my_generic_fragment_forward()

    test_tree_flood();
}
