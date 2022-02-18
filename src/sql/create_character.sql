INSERT INTO `Character` (
	`account_id`,
    `name`,
    `home`,
    `gender`,
    `race`,
    `hair_style`,
    `hair_color`,
    `bank_max`,
    `created_on`
) VALUES (
	:account_id,
    :name,
    :home,
    :gender,
    :race,
    :hair_style,
    :hair_color,
    :bank_max,
    NOW()
);
