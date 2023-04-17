#### Rust

### Configuration Struct
- Stores all configurable things
### Configuration Builder
- Builds the configuration struct

### App Struct
Initializes the application and then runs it

```rust
fn main() {
    let config: Config = ConfigBuilder::new().auth(args.auth).build()
    App::new(config).start()
}
```

