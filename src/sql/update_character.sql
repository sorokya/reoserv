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
    `bank_max` = :bank_max,
    `gold_bank` = :gold_bank,
    `updated_at` = CURRENT_TIMESTAMP()
WHERE `id` = :character_id;
