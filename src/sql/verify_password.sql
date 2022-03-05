SELECT id
FROM Account
WHERE name = :name AND password_hash = :password_hash;