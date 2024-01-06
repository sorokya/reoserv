INSERT INTO `GuildRank` (`guild_id`, `index`, `rank`)
VALUES (:guild_id, 0, :leader_rank_name),
     (:guild_id, 1, :recruiter_rank_name),
     (:guild_id, 2, ''),
     (:guild_id, 3, ''),
     (:guild_id, 4, ''),
     (:guild_id, 5, ''),
     (:guild_id, 6, ''),
     (:guild_id, 7, ''),
		 (:guild_id, 8, :new_member_rank_name);
