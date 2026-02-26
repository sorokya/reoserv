SELECT `rank`
FROM `guilds`
INNER JOIN `guild_ranks`
  ON `guild_ranks`.`guild_id` = `guilds`.`id`
WHERE `tag` = :tag
ORDER BY `index` ASC
