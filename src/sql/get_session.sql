SELECT `account_id`
FROM `Session`
WHERE `token` = :token AND NOW() < DATE_ADD(`created_at`, INTERVAL `ttl` MINUTE);