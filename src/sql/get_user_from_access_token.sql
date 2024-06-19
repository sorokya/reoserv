SELECT `Account`.`id`,
		 `Account`.`name`,
		 MAX(`Character`.`admin_level`) 'admin_level'
FROM `AccessToken`
INNER JOIN `Account`
	ON `Account`.`id` = `AccessToken`.`account_id`
INNER JOIN `Character`
	ON `Character`.`account_id` = `Account`.`id`
WHERE `token` = :access_token
AND TIMESTAMPDIFF(MINUTE, `AccessToken`.`created_at`, NOW()) < `ttl`
GROUP BY `Account`.`id`, `Account`.`name`
