---
inject: true
to: src/player/handlers/<%= family %>/mod.rs
skip_if: <%= action %>
after: 
---

mod <%= action %>;
pub use <%= action %>::<%= action %>;