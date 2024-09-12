CREATE DATABASE IF NOT EXISTS `reoserv`;
USE `reoserv`;

CREATE TABLE IF NOT EXISTS `Account` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(16) NOT NULL,
  `password_hash` char(100) NOT NULL,
  `real_name` varchar(64) NOT NULL,
  `location` varchar(64) NOT NULL,
  `email` varchar(64) NOT NULL,
  `computer` varchar(64) NOT NULL,
  `hdid` int unsigned NOT NULL,
  `register_ip` varchar(15) NOT NULL,
  `last_login_ip` varchar(15) DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `name_UNIQUE` (`name`)
);

CREATE TABLE IF NOT EXISTS `Guild` (
  `id` int NOT NULL AUTO_INCREMENT,
  `tag` varchar(3) NOT NULL,
  `name` varchar(32) NOT NULL,
  `description` text,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `bank` int NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`)
);

CREATE TABLE IF NOT EXISTS `GuildRank` (
  `id` int NOT NULL AUTO_INCREMENT,
  `guild_id` int NOT NULL,
  `index` tinyint NOT NULL,
  `rank` varchar(64) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `guild_rank_guild_id` (`guild_id`),
  CONSTRAINT `guild_rank_guild_id` FOREIGN KEY (`guild_id`) REFERENCES `Guild` (`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `Character` (
  `id` int NOT NULL AUTO_INCREMENT,
  `account_id` int NOT NULL,
  `name` varchar(16) NOT NULL,
  `title` varchar(32) DEFAULT NULL,
  `home` varchar(32) DEFAULT NULL,
  `fiance` varchar(16) DEFAULT NULL,
  `partner` varchar(16) DEFAULT NULL,
  `admin_level` int NOT NULL DEFAULT '0',
  `class` int NOT NULL DEFAULT '1',
  `gender` int NOT NULL DEFAULT '0',
  `race` int NOT NULL DEFAULT '0',
  `hair_style` int NOT NULL DEFAULT '0',
  `hair_color` int NOT NULL DEFAULT '0',
  `bank_level` int NOT NULL DEFAULT '0',
  `gold_bank` int NOT NULL DEFAULT '0',
  `guild_id` int DEFAULT NULL,
  `guild_rank` tinyint DEFAULT NULL,
  `guild_rank_string` varchar(16) DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `name_UNIQUE` (`name`),
  KEY `account_id_idx` (`account_id`),
  KEY `guild_id_idx` (`guild_id`),
  CONSTRAINT `character_account_id` FOREIGN KEY (`account_id`) REFERENCES `Account` (`id`) ON DELETE CASCADE,
  CONSTRAINT `character_guild_id` FOREIGN KEY (`guild_id`) REFERENCES `Guild` (`id`) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS `Ban` (
  `id` int NOT NULL AUTO_INCREMENT,
  `account_id` int NOT NULL,
  `ip` varchar(15) NOT NULL,
  `duration` int DEFAULT NULL,
  `created_by` int NOT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `account_id_idx` (`account_id`),
  KEY `created_by_idx` (`created_by`),
  CONSTRAINT `ban_account_id` FOREIGN KEY (`account_id`) REFERENCES `Account` (`id`) ON DELETE CASCADE,
  CONSTRAINT `ban_created_by` FOREIGN KEY (`created_by`) REFERENCES `Character` (`id`)
);

CREATE TABLE IF NOT EXISTS `Bank` (
  `character_id` int NOT NULL,
  `item_id` int NOT NULL,
  `quantity` int NOT NULL,
  PRIMARY KEY (`character_id`,`item_id`),
  CONSTRAINT `bank_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `BoardPost` (
  `id` int NOT NULL AUTO_INCREMENT,
  `board_id` tinyint NOT NULL,
  `character_id` int NOT NULL,
  `subject` varchar(32) NOT NULL,
  `body` varchar(2048) NOT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  KEY `board_id_idx` (`board_id`),
  KEY `board_post_character_id` (`character_id`),
  CONSTRAINT `board_post_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `Inventory` (
  `character_id` int NOT NULL,
  `item_id` int NOT NULL,
  `quantity` int NOT NULL,
  PRIMARY KEY (`character_id`,`item_id`),
  CONSTRAINT `inventory_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `Paperdoll` (
  `character_id` int NOT NULL,
  `boots` int NOT NULL DEFAULT '0',
  `accessory` int NOT NULL DEFAULT '0',
  `gloves` int NOT NULL DEFAULT '0',
  `belt` int NOT NULL DEFAULT '0',
  `armor` int NOT NULL DEFAULT '0',
  `necklace` int NOT NULL DEFAULT '0',
  `hat` int NOT NULL DEFAULT '0',
  `shield` int NOT NULL DEFAULT '0',
  `weapon` int NOT NULL DEFAULT '0',
  `ring` int NOT NULL DEFAULT '0',
  `ring2` int NOT NULL DEFAULT '0',
  `armlet` int NOT NULL DEFAULT '0',
  `armlet2` int NOT NULL DEFAULT '0',
  `bracer` int NOT NULL DEFAULT '0',
  `bracer2` int NOT NULL DEFAULT '0',
  PRIMARY KEY (`character_id`),
  CONSTRAINT `paperdoll_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `Position` (
  `character_id` int NOT NULL,
  `map` int NOT NULL DEFAULT '192',
  `x` int NOT NULL DEFAULT '7',
  `y` int NOT NULL DEFAULT '6',
  `direction` int NOT NULL DEFAULT '2',
  `sitting` int NOT NULL DEFAULT '0',
  `hidden` int NOT NULL DEFAULT '0',
  PRIMARY KEY (`character_id`),
  CONSTRAINT `position_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `QuestProgress` (
  `character_id` int NOT NULL,
  `quest_id` int NOT NULL,
  `state` int NOT NULL,
  `npc_kills` json NOT NULL,
  `player_kills` int NOT NULL,
  `done_at` datetime DEFAULT NULL,
  `completions` int NOT NULL,
  PRIMARY KEY (`character_id`,`quest_id`),
  CONSTRAINT `quest_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `Spell` (
  `character_id` int NOT NULL,
  `spell_id` int NOT NULL,
  `level` int NOT NULL DEFAULT '0',
  PRIMARY KEY (`character_id`,`spell_id`),
  CONSTRAINT `spell_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `Stats` (
  `character_id` int NOT NULL,
  `level` int NOT NULL DEFAULT '0',
  `experience` int NOT NULL DEFAULT '0',
  `hp` int NOT NULL DEFAULT '10',
  `tp` int NOT NULL DEFAULT '10',
  `strength` int NOT NULL DEFAULT '0',
  `intelligence` int NOT NULL DEFAULT '0',
  `wisdom` int NOT NULL DEFAULT '0',
  `agility` int NOT NULL DEFAULT '0',
  `constitution` int NOT NULL DEFAULT '0',
  `charisma` int NOT NULL DEFAULT '0',
  `stat_points` int NOT NULL DEFAULT '0',
  `skill_points` int NOT NULL DEFAULT '0',
  `karma` int NOT NULL DEFAULT '1000',
  `usage` int NOT NULL DEFAULT '0',
  PRIMARY KEY (`character_id`),
  CONSTRAINT `stats_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS `AutoPickup` (
  `character_id` int NOT NULL,
  `item_id` int NOT NULL,
  PRIMARY KEY (`character_id`, `item_id`),
  CONSTRAINT `autopickup_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE
);
