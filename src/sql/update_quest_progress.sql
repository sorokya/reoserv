UPDATE `QuestProgress`
SET `state` = :state,
    `npc_kills` = :npc_kills,
    `player_kills` = :player_kills,
    `done` = :done
WHERE `character_id` = :character_id AND `quest_id` = :quest_id;