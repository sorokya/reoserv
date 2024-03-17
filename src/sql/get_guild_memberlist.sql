SELECT `Character`.`name`,
       `Character`.`guild_rank`,
       `Character`.`guild_rank_string`
FROM `Guild`
INNER JOIN `Character`
	ON `Character`.`guild_id` = `Guild`.`id`
WHERE :guild_identity IN (`Guild`.`tag`, `Guild`.`name`)
ORDER BY `Character`.`guild_rank`
