SELECT
    c.id,
    c.name,
    s.level,
    c.gender,
    c.hair_style,
    c.hair_color,
    c.race,
    c.admin_level,
    p.boots,
    p.armor,
    p.hat,
    p.shield,
    p.weapon
FROM `Character` c
    INNER JOIN Stats s ON s.character_id = c.id
    INNER JOIN Paperdoll p ON p.character_id = c.id
WHERE
    c.account_id = :account_id;
