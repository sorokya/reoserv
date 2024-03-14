UPDATE `GuildRank`
INNER JOIN `Guild`
  ON `Guild`.id = `GuildRank`.guild_id
SET `rank` = :rank
WHERE `tag` = :tag AND `index` = :index;

