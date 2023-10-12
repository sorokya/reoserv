INSERT INTO `Character` (
	`account_id`,
    `name`,
    `home`,
    `gender`,
    `race`,
    `hair_style`,
    `hair_color`,
    `created_at`
) VALUES (
	:account_id,
    :name,
    :home,
    :gender,
    :race,
    :hair_style,
    :hair_color,
    UTC_TIMESTAMP()
);
