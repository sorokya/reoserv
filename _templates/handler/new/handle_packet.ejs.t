---
inject: true
to: src/player/handle_packet.rs
after: match family {
skip_if: Family::<%= h.capitalize(family) %> => match action
---

        Family::<%= h.capitalize(family) %> => match action {
            Action::<%= h.capitalize(action) %> => {
                handlers::<%= family %>::<%= action %>(buf, player.clone()).await;
            }
            _ => error!("Unhandled packet {:?}_{:?}", action, family),
        },