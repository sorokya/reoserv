UPDATE `accounts`
SET `password_hash` = :password_hash
WHERE `id` = :id;
