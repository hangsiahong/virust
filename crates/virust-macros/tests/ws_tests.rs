use virust_macros::ws;

#[ws]
async fn chat(msg: String) -> String {
    format!("echo: {}", msg)
}

fn main() {}