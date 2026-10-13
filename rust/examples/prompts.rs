//! Prints the consolidation prompts and schemas (conformance: compared with the Python reference).
fn main() {
    use rewire::consolidate::*;
    print!("{}\n=====\n{}\n=====\n{}\n=====\n", extract_system("coding"), extract_system("general"), JUDGE_SYSTEM);
    print!("{}\n=====\n{}\n", rewire::pyjson::dumps_utf8(&extract_schema()), rewire::pyjson::dumps_utf8(&judge_schema()));
}
