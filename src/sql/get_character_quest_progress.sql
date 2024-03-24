SELECT `quest_id`, `state` 
FROM `QuestProgress`
WHERE `character_id` = :character_id;
