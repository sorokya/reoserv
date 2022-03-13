UPDATE `Position`
SET `map` = :map_id,
	`x` = :x,
    `y` = :y,
    `direction` = :direction,
    `sitting` = :sitting,
    `hidden` = :hidden
WHERE `character_id` = :character_id;
