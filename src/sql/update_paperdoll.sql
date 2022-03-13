UPDATE `Paperdoll`
SET `boots` = :boots,
	`accessory` = :accessory,
    `gloves` = :gloves,
    `belt` = :belt,
    `armor` = :armor,
    `necklace` = :necklace,
    `hat` = :hat,
    `shield` = :shield,
    `weapon` = :weapon,
    `ring` = :ring,
    `ring2` = :ring2,
    `armlet` = :armlet,
    `armlet2` = :armlet2,
    `bracer` = :bracer,
    `bracer2` = :bracer2
WHERE `character_id` = :character_id;
