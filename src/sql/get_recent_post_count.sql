SELECT COUNT(1) 'recent_posts'
FROM `BoardPost`
WHERE `board_id` = :board_id
AND `character_id` = :character_id
AND `created_at` > DATE_SUB(NOW(), INTERVAL :post_time MINUTE);
