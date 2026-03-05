use virust_macros::get;

#[get]
pub async fn GET() -> &'static str {
    "test"
}

#[get]
pub async fn POST() -> &'static str {
    "post test"
}

#[get]
pub async fn PUT() -> &'static str {
    "put test"
}

#[get]
pub async fn DELETE() -> &'static str {
    "delete test"
}

#[get]
pub async fn UnknownFunction() -> &'static str {
    "should default to GET"
}
