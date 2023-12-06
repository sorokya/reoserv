INSERT INTO `Ban` (account_id, ip, duration, created_by)
VALUES (:account_id, :ip, :duration, (SELECT id FROM `Character` WHERE `name` = :admin_name));
