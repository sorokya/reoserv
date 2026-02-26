UPDATE `character_spells`
SET level = :level
WHERE character_id = :character_id
AND spell_id = :spell_id;
