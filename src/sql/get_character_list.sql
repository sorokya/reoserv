SELECT
    id,
    name,
    level,
    gender,
    hair_style,
    hair_color,
    race,
    admin_level,
    boots,
    armor,
    hat,
    shield,
    weapon
FROM `characters`
WHERE
    account_id = :account_id;
