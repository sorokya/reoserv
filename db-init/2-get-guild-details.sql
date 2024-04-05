DELIMITER $$

CREATE PROCEDURE `GetGuildDetails`(
	IN `guild_identity` VARCHAR(32)
)
LANGUAGE SQL
NOT DETERMINISTIC
CONTAINS SQL
SQL SECURITY DEFINER
COMMENT ''
BEGIN

SELECT `tag`,
       `name`,
       `description`,
       `created_at`,
       `bank`
FROM `Guild`
WHERE `guild_identity` IN (`tag`, `name`);

SELECT `rank`
FROM `GuildRank`
INNER JOIN `Guild`
	ON `Guild`.id = `GuildRank`.`guild_id`
WHERE `guild_identity` IN (`Guild`.`tag`, `Guild`.`name`)
ORDER BY `guild_id`, `index`
LIMIT 9;

SELECT `Character`.`name`,
		 `Character`.guild_rank
FROM `Guild`
INNER JOIN `Character`
	ON `Character`.`guild_id` = `Guild`.id
	AND `Character`.`guild_rank` <= 2
WHERE `guild_identity` IN (`Guild`.`tag`, `Guild`.`name`);

END $$
