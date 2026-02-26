DELETE FROM `character_spells`
WHERE character_id = :character_id
AND spell_id = :spell_id;
