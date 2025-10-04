use exum::*;

#[main]
async fn main() {
    app.static_("/", ".");
}