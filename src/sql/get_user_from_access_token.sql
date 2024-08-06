SELECT `Account`.`id` 'account_id',
		 `Account`.`name` 'account_name',
     `Character`.`id` 'character_id',
     `Character`.`name` 'character_name',
		 `Character`.`admin_level` 'admin_level'
FROM `AccessToken`
INNER JOIN `Account`
	ON `Account`.`id` = `AccessToken`.`account_id`
LEFT JOIN `Character`
	ON `Character`.`account_id` = `Account`.`id`
WHERE `token` = :access_token
AND TIMESTAMPDIFF(MINUTE, `AccessToken`.`created_at`, NOW()) < `ttl`
