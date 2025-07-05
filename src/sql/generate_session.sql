INSERT INTO `Session` (`account_id`, `token`)
VALUES (:account_id, :token)
ON DUPLICATE KEY UPDATE `token` = :token, created_at = CURRENT_TIMESTAMP;