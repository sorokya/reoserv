INSERT INTO `GuildRank` (`guild_id`, `rank`)
VALUES (:guild_id, :leader_rank_name),
     (:guild_id, :recruiter_rank_name),
     (:guild_id, ''),
     (:guild_id, ''),
     (:guild_id, ''),
     (:guild_id, ''),
     (:guild_id, ''),
     (:guild_id, ''),
		 (:guild_id, :new_member_rank_name);
