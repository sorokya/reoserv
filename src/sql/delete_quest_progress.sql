DELETE FROM `character_quest_progress`
WHERE `character_id` = :character_id AND `quest_id` = :quest_id
