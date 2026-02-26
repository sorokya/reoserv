SELECT a.id, h.ip
FROM `characters` c
INNER JOIN `accounts` a
	ON a.id = c.account_id
INNER JOIN (
	SELECT `account_id`, MAX(`id`) AS `history_id`
	FROM `history`
	GROUP BY `account_id`
) latest_history ON latest_history.account_id = a.id
INNER JOIN `history` h
	ON h.id = latest_history.history_id
AND h.account_id = latest_history.account_id
WHERE c.name = :character_name;
