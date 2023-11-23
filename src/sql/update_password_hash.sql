UPDATE `Account` 
SET `password_hash` = :password_hash
WHERE `id` = :id;
