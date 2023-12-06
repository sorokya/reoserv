SELECT a.id, a.last_login_ip
FROM `Character` c
INNER JOIN `Account` a
	ON a.id = c.account_id
WHERE c.name = :character_name;
