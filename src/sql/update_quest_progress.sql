UPDATE `QuestProgress`
SET `state` = :state
WHERE `character_id` = :character_id AND `quest_id` = :quest_id;
