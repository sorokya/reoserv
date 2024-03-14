SELECT `rank` 
FROM `Guild` 
INNER JOIN `GuildRank` 
  ON `GuildRank`.`guild_id` = `Guild`.`id`
WHERE `tag` = :tag
ORDER BY `index` ASC
