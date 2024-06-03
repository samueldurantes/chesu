use std::env;

fn main() {
    let cwd = env::current_dir().unwrap();
    let path = env::args()
        .nth(1)
        .map(|p| cwd.join(p))
        .expect("missing path to json file");

    let (_, open_api) = server::app::make_app();
    let json = serde_json::to_string_pretty(&open_api).unwrap();

    std::fs::write(path, json).unwrap();

    println!("OpenAPI schema generated successfully!");
}
