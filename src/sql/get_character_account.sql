SELECT a.id, h.ip
FROM `characters` c
INNER JOIN `accounts` a
	ON a.id = c.account_id
INNER JOIN (
	SELECT `account_id`, MAX(`ip`) AS `ip`
	FROM `history`
	GROUP BY `account_id`
) h ON a.id = h.account_id
WHERE c.name = :character_name;
