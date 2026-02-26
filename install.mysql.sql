CREATE TABLE
    IF NOT EXISTS `accounts` (
        `id` INTEGER NOT NULL AUTO_INCREMENT,
        `name` TEXT NOT NULL UNIQUE,
        `password_hash` TEXT NOT NULL,
        `email` TEXT NOT NULL,
        `real_name` TEXT NOT NULL,
        `location` TEXT NOT NULL,
        `computer` TEXT NOT NULL,
        `hdid` INTEGER UNSIGNED NOT NULL,
        `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CONSTRAINT `accounts_name_unique` UNIQUE (`name`),
        PRIMARY KEY (`id`)
    );

CREATE TABLE
    IF NOT EXISTS `account_sessions` (
        `id` INTEGER NOT NULL AUTO_INCREMENT,
        `account_id` INTEGER NOT NULL,
        `token` VARCHAR(100) NOT NULL,
        `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        `ttl` INTEGER NOT NULL DEFAULT 60,
        FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE,
        PRIMARY KEY (`id`)
    );

CREATE TABLE
    IF NOT EXISTS `bans` (
        `account_id` INTEGER,
        `ip` VARCHAR(15),
        `duration` INTEGER,
        `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS `guilds` (
        `id` INTEGER NOT NULL AUTO_INCREMENT,
        `tag` VARCHAR(3) NOT NULL,
        `name` VARCHAR(32) NOT NULL,
        `description` TEXT,
        `bank` INTEGER NOT NULL DEFAULT 0,
        `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        CONSTRAINT `guilds_tag_unique` UNIQUE (`tag`),
        CONSTRAINT `guilds_name_unique` UNIQUE (`name`),
        PRIMARY KEY (`id`)
    );

CREATE TABLE
    IF NOT EXISTS `guild_ranks` (
        `id` INTEGER NOT NULL AUTO_INCREMENT,
        `guild_id` INTEGER NOT NULL,
        `index` TINYINT NOT NULL,
        `rank` VARCHAR(64) NOT NULL,
        FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`) ON DELETE CASCADE,
        PRIMARY KEY (`id`)
    );

CREATE TABLE
    IF NOT EXISTS `characters` (
        `id` INTEGER NOT NULL AUTO_INCREMENT,
        `account_id` INTEGER NOT NULL,
        `name` VARCHAR(16) NOT NULL,
        `map` INTEGER NOT NULL DEFAULT 192,
        `x` INTEGER NOT NULL DEFAULT 7,
        `y` INTEGER NOT NULL DEFAULT 6,
        `direction` INTEGER NOT NULL DEFAULT 2,
        `sitting` INTEGER NOT NULL DEFAULT 0,
        `hidden` INTEGER NOT NULL DEFAULT 0,
        `title` VARCHAR(32),
        `home` VARCHAR(32),
        `fiance` VARCHAR(16),
        `partner` VARCHAR(16),
        `admin_level` INTEGER NOT NULL DEFAULT 0,
        `class` INTEGER NOT NULL DEFAULT 1,
        `gender` INTEGER NOT NULL DEFAULT 0,
        `race` INTEGER NOT NULL DEFAULT 0,
        `hair_style` INTEGER NOT NULL DEFAULT 0,
        `hair_color` INTEGER NOT NULL DEFAULT 0,
        `bank_level` INTEGER NOT NULL DEFAULT 0,
        `gold_bank` INTEGER NOT NULL DEFAULT 0,
        `guild_id` INTEGER,
        `guild_rank` TINYINT,
        `guild_rank_string` VARCHAR(16),
        `level` INTEGER NOT NULL DEFAULT 0,
        `experience` INTEGER NOT NULL DEFAULT 0,
        `hp` INTEGER NOT NULL DEFAULT 10,
        `tp` INTEGER NOT NULL DEFAULT 10,
        `strength` INTEGER NOT NULL DEFAULT 0,
        `intelligence` INTEGER NOT NULL DEFAULT 0,
        `wisdom` INTEGER NOT NULL DEFAULT 0,
        `agility` INTEGER NOT NULL DEFAULT 0,
        `constitution` INTEGER NOT NULL DEFAULT 0,
        `charisma` INTEGER NOT NULL DEFAULT 0,
        `stat_points` INTEGER NOT NULL DEFAULT 0,
        `skill_points` INTEGER NOT NULL DEFAULT 0,
        `karma` INTEGER NOT NULL DEFAULT 1000,
        `usage` INTEGER NOT NULL DEFAULT 0,
        `boots` int NOT NULL DEFAULT 0,
        `accessory` int NOT NULL DEFAULT 0,
        `gloves` int NOT NULL DEFAULT 0,
        `belt` int NOT NULL DEFAULT 0,
        `armor` int NOT NULL DEFAULT 0,
        `necklace` int NOT NULL DEFAULT 0,
        `hat` int NOT NULL DEFAULT 0,
        `shield` int NOT NULL DEFAULT 0,
        `weapon` int NOT NULL DEFAULT 0,
        `ring` int NOT NULL DEFAULT 0,
        `ring2` int NOT NULL DEFAULT 0,
        `armlet` int NOT NULL DEFAULT 0,
        `armlet2` int NOT NULL DEFAULT 0,
        `bracer` int NOT NULL DEFAULT 0,
        `bracer2` int NOT NULL DEFAULT 0,
        `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`) ON DELETE CASCADE,
        FOREIGN KEY (`guild_id`) REFERENCES `guilds` (`id`) ON DELETE SET NULL,
        PRIMARY KEY (`id`)
    );

CREATE TABLE
    IF NOT EXISTS `character_bank` (
        `character_id` INTEGER NOT NULL,
        `item_id` INTEGER NOT NULL,
        `quantity` INTEGER NOT NULL DEFAULT 1,
        PRIMARY KEY (`character_id`, `item_id`),
        FOREIGN KEY (`character_id`) REFERENCES `characters` (`id`) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS `character_inventory` (
        `character_id` INTEGER NOT NULL,
        `item_id` INTEGER NOT NULL,
        `quantity` INTEGER NOT NULL DEFAULT 1,
        PRIMARY KEY (`character_id`, `item_id`),
        FOREIGN KEY (`character_id`) REFERENCES `characters` (`id`) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS `character_quest_progress` (
        `character_id` INTEGER NOT NULL,
        `quest_id` INTEGER NOT NULL,
        `state` INTEGER NOT NULL,
        `npc_kills` TEXT NOT NULL,
        `player_kills` INTEGER NOT NULL,
        `done_at` DATETIME,
        `completions` INTEGER NOT NULL,
        PRIMARY KEY (`character_id`, `quest_id`),
        FOREIGN KEY (`character_id`) REFERENCES `characters` (`id`) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS `character_spells` (
        `character_id` INTEGER NOT NULL,
        `spell_id` INTEGER NOT NULL,
        `level` INTEGER NOT NULL DEFAULT 0,
        PRIMARY KEY (`character_id`, `spell_id`),
        FOREIGN KEY (`character_id`) REFERENCES `characters` (`id`) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS `board_posts` (
        `id` INTEGER NOT NULL AUTO_INCREMENT,
        `board_id` TINYINT NOT NULL,
        `character_id` INTEGER NOT NULL,
        `subject` VARCHAR(32) NOT NULL,
        `body` VARCHAR(2048) NOT NULL,
        `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (`character_id`) REFERENCES `characters` (`id`) ON DELETE CASCADE,
        PRIMARY KEY (`id`)
    );

CREATE TABLE
    IF NOT EXISTS `character_auto_pickup` (
        `character_id` int NOT NULL,
        `item_id` int NOT NULL,
        PRIMARY KEY (`character_id`, `item_id`),
        FOREIGN KEY (`character_id`) REFERENCES `characters` (`id`) ON DELETE CASCADE
    );

CREATE TABLE
    IF NOT EXISTS `history` (
        `id` INTEGER NOT NULL AUTO_INCREMENT,
        `account_id` INTEGER NOT NULL,
        `character_id` INTEGER,
        `action_id` INTEGER NOT NULL,
        `ip` VARCHAR(15) NOT NULL,
        `timestamp` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        `details` TEXT,
        FOREIGN KEY (`account_id`) REFERENCES `accounts` (`id`),
        FOREIGN KEY (`character_id`) REFERENCES `characters` (`id`),
        PRIMARY KEY (`id`)
    );

CREATE INDEX IF NOT EXISTS `idx_characters_account_id` ON `characters` (`account_id`);

CREATE INDEX IF NOT EXISTS `idx_characters_guild_id` ON `characters` (`guild_id`);

CREATE INDEX IF NOT EXISTS `idx_character_bank_character_id` ON `character_bank` (`character_id`);

CREATE INDEX IF NOT EXISTS `idx_character_inventory_character_id` ON `character_inventory` (`character_id`);

CREATE INDEX IF NOT EXISTS `idx_character_quest_progress_character_id` ON `character_quest_progress` (`character_id`);

CREATE INDEX IF NOT EXISTS `idx_character_spells_character_id` ON `character_spells` (`character_id`);

CREATE INDEX IF NOT EXISTS `idx_bans_account_id` ON `bans` (`account_id`);

CREATE INDEX IF NOT EXISTS `idx_bans_ip` ON `bans` (`ip`);