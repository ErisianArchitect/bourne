A simple JSON library written entirely in Rust.

Use `preserve_order` feature to preserve element order in `Value::Object(_)`

```rust
use std::str::FromStr;

use bourne::{
    Value,
    json
};

fn main() -> Result<(), bourne::error::ParseError> {
    let number = 3.14;
    let value = json!(
        {
            "example" : "Hello, world!",
            "number" : number,
        }
    );
    println!("{value}");
    println!("################################");
    let value = Value::from_str(r#"
        {
            "integer" : 123,
            "decimal" : [3.14, 1.0],
            "keywords" : [true, false, null],
            "nested" : {
                "one" : 1,
                "two" : {
                    "three" : 3
                }
            }
        }
    "#)?;
    println!("{value}");
    println!("################################");
    println!("{}", value.to_string_compressed());
    Ok(())
}
```