# gramic

Telegram webhook in Rust.

```toml
[dependencies]
gramic = "0.1"
tokio  = { version = "1", features = ["full"] }
```

## Usage

```rust
// receive updates
gramic::serve("TOKEN", "https://yourserver.com", |u| async move {
    println!("{:?}", u.message);
}).await.unwrap();

// set webhook
gramic::set("TOKEN", "https://yourserver.com").await?;

// delete webhook
gramic::delete("TOKEN").await?;

// check status
let info = gramic::info("TOKEN").await?;
```

## Custom config

```rust
Bot::new("TOKEN", "https://yourserver.com")
    .port(8443)
    .path("/updates")
    .secret("some_secret")
    .serve(|u| async move { println!("{:?}", u.message); })
    .await?;
```

## Run the example

```sh
cp .env.example .env

cargo run --example bot -- serve
cargo run --example bot -- set
cargo run --example bot -- delete
cargo run --example bot -- info
```
