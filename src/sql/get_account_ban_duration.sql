SELECT IFNULL(b.`duration`, 0) 'duration', b.`created_at`
FROM `Ban` b
INNER JOIN `Account` a
  ON a.id = b.account_id
  AND a.name = :name
ORDER BY b.`created_at` DESC
