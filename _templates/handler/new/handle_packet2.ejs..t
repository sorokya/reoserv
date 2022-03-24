---
inject: true
to: src/player/handle_packet.rs
after: Family::<%= h.capitalize(family) %> => match action {
skip_if: Action::<%= h.capitalize(action) %> => {
---

            Action::<%= h.capitalize(action) %> => {
                handlers::<%= family %>::<%= action %>(buf, player.clone()).await;
            }