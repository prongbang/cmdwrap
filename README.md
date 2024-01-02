# cmdwrap

How to use:

- synchronous

```rust
let command = "pwd";
match cmdwrap::run(command) {
    Ok(output) => {
        println!("{}", output)
    }
    Err(error) => {
        println!("\tCommand execution failed:\n{}", error);
    }
}
```

- asynchronous

```rust
use futures_util::pin_mut;
use futures_util::stream::StreamExt;

let command = "pwd";
let mut s = cmdwrap::run_stream(command);
pin_mut!(s); // needed for iteration
while let Some(value) = s.next().await {
    println!("{}", value.output);
}
```