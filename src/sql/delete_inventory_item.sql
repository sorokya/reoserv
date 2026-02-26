DELETE FROM `character_inventory`
WHERE character_id = :character_id
AND item_id = :item_id;