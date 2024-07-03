SELECT `Account`.`id`,
		 `Account`.`name`,
     `Account`.`real_name`,
     `Account`.`location`,
     `Account`.`email`
FROM `Account`
WHERE `Account`.`id` = :id
