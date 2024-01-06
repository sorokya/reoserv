UPDATE `Character`
SET `title` = :title,
	`home` = :home,
    `fiance` = :fiance,
    `partner` = :partner,
    `admin_level` = :admin_level,
    `class` = :class,
    `gender` = :gender,
    `race` = :race,
    `hair_style` = :hair_style,
    `hair_color` = :hair_color,
    `bank_level` = :bank_level,
    `gold_bank` = :gold_bank,
    `guild_id` = (SELECT `id` FROM `Guild` WHERE `tag` = :guild_tag),
    `guild_rank_index` = :guild_rank_index,
    `guild_rank_string` = :guild_rank_string,
    `updated_at` = CURRENT_TIMESTAMP()
WHERE `id` = :character_id;
