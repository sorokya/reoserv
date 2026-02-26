SELECT COUNT(1)
FROM `characters`
INNER JOIN `guilds`
	ON `guilds`.`id` = `characters`.`guild_id`
	AND `guilds`.`tag` = :guild_tag
WHERE `characters`.`guild_rank` = 1;
