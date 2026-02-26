SELECT `characters`.`name`,
       `characters`.`guild_rank`,
       `characters`.`guild_rank_string`
FROM `guilds`
INNER JOIN `characters`
	ON `characters`.`guild_id` = `guilds`.`id`
WHERE :guild_identity IN (`guilds`.`tag`, `guilds`.`name`)
ORDER BY `characters`.`guild_rank`
