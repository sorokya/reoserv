INSERT INTO `Character` (
	`account_id`,
    `name`,
    `home`,
    `gender`,
    `race`,
    `hair_style`,
    `hair_color`,
    `bank_max`,
    `created_at`
) VALUES (
	:account_id,
    :name,
    :home,
    :gender,
    :race,
    :hair_style,
    :hair_color,
    :bank_max,
    UTC_TIMESTAMP()
);
