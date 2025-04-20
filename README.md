![giphy](https://github.com/user-attachments/assets/de31c945-708b-43b3-908e-2d32c6f98d5d)

A simple JSON library written entirely in Rust.

Use `preserve_order` feature to preserve element order in `Value::Object(_)`. This will use `indexmap`, which will incur a significant memory overhead.

```rust
use std::str::FromStr;

use bourne::{
    Value,
    json
};
use bourne::format::Indent;
use bourne::error::ParseError;

fn main() -> Result<(), ParseError> {
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
    println!("{}", value.pretty_print());
    println!("################################");
    println!("{}", value.pretty_print_format(Indent::Tabs(1), true));
    Ok(())
}
```
