#[macro_use]
extern crate cucumber_rust;

use std::fs;

pub struct MyWorld {
    // You can use this struct for mutable context in scenarios.
    foo: String
}

impl cucumber_rust::World for MyWorld {}
impl std::default::Default for MyWorld {
    fn default() -> MyWorld {
        // This function is called every time a new scenario is started
        MyWorld { 
            foo: "a default string".to_string()
        }
    }
}

mod default_steps {
    // Any type that implements cucumber_rust::World + Default can be the world
    steps!(::MyWorld => {
        given "Music is in \"unsorted\" directory" |_world, _step| {
            //check unsorted directroy exists
            let unsorted_path = Path::new("unittest-unsorted");
            if !unsorted_path.exists() {
                match std::fs::create_dir_all("unittest-unsorted") {
                    Ok(o) => o,
                    Err(_e) => panic!("create_dir_all failed with an error when creating unittest-unsorted folder")
                };
            }
            assert_eq!(unsorted_path.exists(), true);
        };

        when "I consider what I am doing" |world, _step| {
            // Take actions
            let new_string = format!("{}.", &world.foo);
            world.foo = new_string;
        };

        then "I am interested in ATDD" |world, _step| {
            // Check that the outcomes to be observed have occurred
            assert_eq!(world.foo, "a default string");
        };

        then regex r"^we can (.*) rules with regex$" |_world, matches, _step| {
            // And access them as an array
            assert_eq!(matches[1], "implement");
        };

        then regex r"^we can also match (\d+) (.+) types$" (usize, String) |_world, num, word, _step| {
            // `num` will be of type usize, `word` of type String
            assert_eq!(num, 42);
            assert_eq!(word, "olika");
        };
    });
}

// Declares a before handler function named `a_before_fn`
before!(a_before_fn => |_scenario| {

});

// Declares an after handler function named `an_after_fn`
after!(an_after_fn => |_scenario| {

});

// A setup function to be called before everything else
fn setup() {
    
}

cucumber! {
    features: "./features", // Path to our feature files
    world: ::MyWorld, // The world needs to be the same for steps and the main cucumber call
    steps: &[
        default_steps::steps // the `steps!` macro creates a `steps` function in a module
    ],
    setup: setup, // Optional; called once before everything
    before: &[
        a_before_fn // Optional; called before each scenario
    ], 
    after: &[
        an_after_fn // Optional; called after each scenario
    ] 
}