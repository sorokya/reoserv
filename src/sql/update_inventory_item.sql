UPDATE Inventory
SET quantity = :quantity
WHERE character_id = :character_id
AND item_id = :item_id;