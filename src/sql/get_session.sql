SELECT `account_id`, `created_at`, `ttl`
FROM `account_sessions`
WHERE `token` = :token
ORDER BY `created_at` DESC
LIMIT 1;