# estimate_size

A library used on iterators which allows adapting a custom size_hint with your estimate, for iterators that do not provide accurate estimates themselves.

# Usage

```rust
use estimate_size::EstimateSize;
(0..10).filter(|x| x % 2 == 0).estimate_exact_size(5)
```
