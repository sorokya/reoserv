INSERT INTO `characters` (
	`account_id`,
    `name`,
    `home`,
    `gender`,
    `race`,
    `hair_style`,
    `hair_color`,
    `map`,
    `x`,
    `y`,
    `direction`,
    `admin_level`
) VALUES (
	:account_id,
    :name,
    :home,
    :gender,
    :race,
    :hair_style,
    :hair_color,
    :map,
    :x,
    :y,
    :direction,
    :admin_level
);
