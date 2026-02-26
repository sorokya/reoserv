INSERT INTO
    `accounts` (
        `id`,
        `name`,
        `password_hash`,
        `real_name`,
        `location`,
        `email`,
        `computer`,
        `hdid`,
        `created_at`
    )
SELECT
    `id`,
    `name`,
    `password_hash`,
    `real_name`,
    `location`,
    `email`,
    `computer`,
    `hdid`,
    `created_at`
FROM
    `Account`;

INSERT INTO
    `account_sessions` (`account_id`, `token`, `created_at`, `ttl`)
SELECT
    `account_id`,
    `token`,
    `created_at`,
    `ttl`
FROM
    `Session`;

INSERT INTO
    `bans` (`account_id`, `ip`, `duration`, `created_at`)
SELECT
    `account_id`,
    `ip`,
    `duration`,
    `created_at`
FROM
    `Ban`;

INSERT INTO
    `guilds` (
        `id`,
        `tag`,
        `name`,
        `description`,
        `bank`,
        `created_at`
    )
SELECT
    `id`,
    `tag`,
    `name`,
    `description`,
    `bank`,
    `created_at`
FROM
    `Guild`;

INSERT INTO
    `guild_ranks` (`id`, `guild_id`, `index`, `rank`)
SELECT
    `id`,
    `guild_id`,
    `index`,
    `rank`
FROM
    `GuildRank`;

INSERT INTO
    `characters` (
        `id`,
        `account_id`,
        `name`,
        `map`,
        `x`,
        `y`,
        `direction`,
        `sitting`,
        `hidden`,
        `title`,
        `home`,
        `fiance`,
        `partner`,
        `admin_level`,
        `class`,
        `gender`,
        `race`,
        `hair_style`,
        `hair_color`,
        `bank_level`,
        `gold_bank`,
        `guild_id`,
        `guild_rank`,
        `guild_rank_string`,
        `level`,
        `experience`,
        `hp`,
        `tp`,
        `strength`,
        `intelligence`,
        `wisdom`,
        `agility`,
        `constitution`,
        `charisma`,
        `stat_points`,
        `skill_points`,
        `karma`,
        `usage`,
        `boots`,
        `accessory`,
        `gloves`,
        `belt`,
        `armor`,
        `necklace`,
        `hat`,
        `shield`,
        `weapon`,
        `ring`,
        `ring2`,
        `armlet`,
        `armlet2`,
        `bracer`,
        `bracer2`,
        `created_at`
    )
SELECT
    `c`.`id`,
    `c`.`account_id`,
    `c`.`name`,
    `p`.`map`,
    `p`.`x`,
    `p`.`y`,
    `p`.`direction`,
    `p`.`sitting`,
    `p`.`hidden`,
    `c`.`title`,
    `c`.`home`,
    `c`.`fiance`,
    `c`.`partner`,
    `c`.`admin_level`,
    `c`.`class`,
    `c`.`gender`,
    `c`.`race`,
    `c`.`hair_style`,
    `c`.`hair_color`,
    `c`.`bank_level`,
    `c`.`gold_bank`,
    `c`.`guild_id`,
    `c`.`guild_rank`,
    `c`.`guild_rank_string`,
    `s`.`level`,
    `s`.`experience`,
    `s`.`hp`,
    `s`.`tp`,
    `s`.`strength`,
    `s`.`intelligence`,
    `s`.`wisdom`,
    `s`.`agility`,
    `s`.`constitution`,
    `s`.`charisma`,
    `s`.`stat_points`,
    `s`.`skill_points`,
    `s`.`karma`,
    `s`.`usage`,
    `d`.`boots`,
    `d`.`accessory`,
    `d`.`gloves`,
    `d`.`belt`,
    `d`.`armor`,
    `d`.`necklace`,
    `d`.`hat`,
    `d`.`shield`,
    `d`.`weapon`,
    `d`.`ring`,
    `d`.`ring2`,
    `d`.`armlet`,
    `d`.`armlet2`,
    `d`.`bracer`,
    `d`.`bracer2`,
    `c`.`created_at`
FROM
    `Character` `c`
    INNER JOIN `Stats` `s` ON `c`.`id` = `s`.`character_id`
    INNER JOIN `Position` `p` ON `c`.`id` = `p`.`character_id`
    INNER JOIN `Paperdoll` `d` ON `c`.`id` = `d`.`character_id`;

INSERT INTO
    `character_auto_pickup` (`character_id`, `item_id`)
SELECT
    `character_id`,
    `item_id`
FROM
    `AutoPickup`;

INSERT INTO
    `character_bank` (`character_id`, `item_id`, `quantity`)
SELECT
    `character_id`,
    `item_id`,
    `quantity`
FROM
    `Bank`;

INSERT INTO
    `character_inventory` (`character_id`, `item_id`, `quantity`)
SELECT
    `character_id`,
    `item_id`,
    `quantity`
FROM
    `Inventory`;

INSERT INTO
    `character_quest_progress` (
        `character_id`,
        `quest_id`,
        `state`,
        `npc_kills`,
        `player_kills`,
        `done_at`,
        `completions`
    )
SELECT
    `character_id`,
    `quest_id`,
    `state`,
    `npc_kills`,
    `player_kills`,
    `done_at`,
    `completions`
FROM
    `QuestProgress`;

INSERT INTO
    `character_spells` (`character_id`, `spell_id`, `level`)
SELECT
    `character_id`,
    `spell_id`,
    `level`
FROM
    `Spell`;

INSERT INTO
    `board_posts` (
        `id`,
        `board_id`,
        `character_id`,
        `subject`,
        `body`,
        `created_at`
    )
SELECT
    `id`,
    `board_id`,
    `character_id`,
    `subject`,
    `body`,
    `created_at`
FROM
    `BoardPost`;

DROP TABLE IF EXISTS `AutoPickup`;

DROP TABLE IF EXISTS `Bank`;

DROP TABLE IF EXISTS `BoardPost`;

DROP TABLE IF EXISTS `Inventory`;

DROP TABLE IF EXISTS `Paperdoll`;

DROP TABLE IF EXISTS `Position`;

DROP TABLE IF EXISTS `QuestProgress`;

DROP TABLE IF EXISTS `Session`;

DROP TABLE IF EXISTS `Spell`;

DROP TABLE IF EXISTS `Stats`;

DROP TABLE IF EXISTS `Ban`;

DROP TABLE IF EXISTS `Character`;

DROP TABLE IF EXISTS `GuildRank`;

DROP TABLE IF EXISTS `Guild`;

DROP TABLE IF EXISTS `Account`;