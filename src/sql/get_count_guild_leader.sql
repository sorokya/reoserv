SELECT COUNT(1)
FROM `Character`
INNER JOIN `Guild`
	ON `Guild`.`id` = `Character`.`guild_id`
	AND `Guild`.`tag` = :guild_tag
WHERE `Character`.`guild_rank` = 1;
