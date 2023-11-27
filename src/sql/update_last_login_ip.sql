UPDATE `Account`
SET `last_login_ip` = :ip,
	  `updated_at` = NOW()
WHERE `id` = :account_id;
