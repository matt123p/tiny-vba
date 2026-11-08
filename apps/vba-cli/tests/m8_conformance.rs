#[allow(dead_code)]
#[path = "../examples/m8_conformance.rs"]
mod runner;
#[test]
fn reviewed_additional_library_observations_match_exact_types_and_values()
-> Result<(), Box<dyn std::error::Error>> {
    runner::run()
}
