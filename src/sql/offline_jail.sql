UPDATE `Position` p
INNER JOIN `Character` c
	ON c.`id` = p.`character_id`
SET p.`map` = :map, p.`x` = :x, p.`y` = :y
WHERE c.`name` = :name;
