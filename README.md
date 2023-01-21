## Archived

This repository is archived to reflect its level of (in)activity and set maintenance expectations.

If some ideas or code in it are useful to you, feel free to use them in other repositories and crates
in accordance with the license.

Note however that tree data structure design in Rust is full of trade-offs,
maybe some approach other than `Rc`/`Weak` would work better for you.
(For example [`Vec` + indices][1], if it’s acceptable
not to recover memory for dropped nodes before the entire document is dropped.)

[1]: https://github.com/SimonSapin/victor/blob/fdb11f3e87f6d2d59170d10169fa6deb94e53b94/victor/src/dom/mod.rs#L19-L29
