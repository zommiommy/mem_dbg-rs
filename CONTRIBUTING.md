# Contributing

If you want to contribute to this project, please follow the Rust development
guidelines at <https://github.com/vigna/rust-dev-guidelines/blob/main/README.md>

## Testing

We use [insta](https://github.com/mitsuhiko/insta) for snapshot testing of the output format. To run these tests:

```bash
cargo test --test test_all_types_mem_dbg_insta
```

If you make changes that affect the output format, you can update the snapshots by running:

```bash
INSTA_UPDATE=always cargo test --test test_all_types_mem_dbg_insta
```
