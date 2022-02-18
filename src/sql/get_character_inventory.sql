SELECT item_id, quantity
FROM Inventory
WHERE character_id = :character_id;
