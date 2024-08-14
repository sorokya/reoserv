UPDATE `Character`
SET `guild_rank` = NULL, `guild_rank_string` = NULL
WHERE `guild_id` IS NULL;
