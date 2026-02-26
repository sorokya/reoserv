SELECT TOP 1 `account_id`
FROM `account_sessions`
WHERE `token` = :token AND CURRENT_TIMESTAMP < DATE_ADD(`created_at`, INTERVAL `ttl` MINUTE);