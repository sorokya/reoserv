SELECT IFNULL(`duration`, 0) 'duration', `created_at`
FROM `bans`
WHERE `ip` = :ip
ORDER BY `created_at` DESC
