# Clean Code Review Report

## Scope

- Files:
- Change:
- Reviewer:

## Summary

...

## Findings

### [Severity] file:line - Title

**Problem:**

...

**Impact:**

...

**Suggested fix:**

...

```rust
// Optional example
```

---

## Function Check

- [ ] Guard clause di awal.
- [ ] Setiap fungsi maksimal 20 baris.
- [ ] Fungsi dipecah menjadi langkah kecil dan reusable.
- [ ] Fungsi existing sudah dicari dan digunakan ulang.
- [ ] Tidak ada duplikat dekat.

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

## Follow-ups

- ...

## Notes

- ...
