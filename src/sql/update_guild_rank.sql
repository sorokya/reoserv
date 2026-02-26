UPDATE `guild_ranks`
INNER JOIN `guilds`
  ON `guilds`.id = `guild_ranks`.guild_id
SET `rank` = :rank
WHERE `tag` = :tag AND `index` = :index;

