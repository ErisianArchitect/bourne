A `json` macro for the [bourne](https://github.com/ErisianArchitect/bourne) JSON library.

#### Example

```rust
json!(
    {
        "int": 1234,
        "float": 3.14,
        "bool": true,
        "null": null,
        "string": "hello, world",
        "array": [1, 2, 3],
        "object": {
            "one": 1,
            "two": 2,
            "three": 3
        }
    }
)
```