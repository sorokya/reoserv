SELECT b.`id`,
	    c.`name` 'author',
	    b.`subject`,
      b.`created_at`
FROM `BoardPost` b
INNER JOIN `Character` c
	ON c.`id` = b.`character_id`
WHERE b.`board_id` = :board_id
ORDER BY b.`id` DESC LIMIT :limit;
