SELECT COUNT(1) 'recent_posts'
FROM `board_posts`
WHERE `board_id` = :board_id
AND `character_id` = :character_id
AND `created_at` > :cutoff_time;
