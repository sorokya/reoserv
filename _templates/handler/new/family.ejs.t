---
inject: true
to: src/player/handlers/mod.rs
after: 
skip_if: <%= family %>
---

pub mod <%= family %>;