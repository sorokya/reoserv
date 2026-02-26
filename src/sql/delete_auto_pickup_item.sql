DELETE FROM `character_auto_pickup`
WHERE character_id = :character_id
AND item_id = :item_id;
