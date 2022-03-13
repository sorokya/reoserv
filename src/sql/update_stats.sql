UPDATE `Stats`
SET `level` = :level,
	`experience` = :experience,
    `hp` = :hp,
    `tp` = :tp,
    `strength` = :strength,
    `intelligence` = :intelligence,
    `wisdom` = :wisdom,
    `agility` = :agility,
    `constitution` = :constitution,
    `charisma` = :charisma,
    `stat_points` = :stat_points,
    `skill_points` = :skill_points,
    `karma` = :karma,
    `usage` = :usage
WHERE `character_id` = :character_id;
