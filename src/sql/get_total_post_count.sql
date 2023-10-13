SELECT COUNT(1) 'total_posts'
FROM (
	SELECT `id`, `character_id`
	FROM `BoardPost`
	WHERE `board_id` = :board_id
	ORDER BY `id` DESC LIMIT :limit
) `a`
WHERE `a`.`character_id` = :character_id;
