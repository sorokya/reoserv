CREATE DATABASE  IF NOT EXISTS `reoserv` /*!40100 DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci */ /*!80016 DEFAULT ENCRYPTION='N' */;
USE `reoserv`;
-- MySQL dump 10.13  Distrib 8.0.22, for macos10.15 (x86_64)
--
-- Host: 127.0.0.1    Database: reoserv
-- ------------------------------------------------------
-- Server version	8.0.28

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!50503 SET NAMES utf8 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `Account`
--

DROP TABLE IF EXISTS `Account`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Account` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(16) NOT NULL,
  `password_hash` char(100) NOT NULL,
  `real_name` varchar(64) NOT NULL,
  `location` varchar(64) NOT NULL,
  `email` varchar(64) NOT NULL,
  `computer` varchar(64) NOT NULL,
  `hdid` int unsigned NOT NULL,
  `register_ip` varchar(15) NOT NULL,
  `last_login_ip` varchar(15) NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `name_UNIQUE` (`name`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `Bank`
--

DROP TABLE IF EXISTS `Bank`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Bank` (
  `character_id` int NOT NULL,
  `item_id` int NOT NULL,
  `quantity` int NOT NULL,
  PRIMARY KEY (`character_id`,`item_id`),
  CONSTRAINT `bank_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `Character`
--

DROP TABLE IF EXISTS `Character`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Character` (
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
  `bank_max` int NOT NULL DEFAULT '0',
  `gold_bank` int NOT NULL DEFAULT '0',
  `guild_id` int DEFAULT NULL,
  `guild_rank_id` int DEFAULT NULL,
  `guild_rank_string` varchar(16) DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` datetime DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `name_UNIQUE` (`name`),
  KEY `account_id_idx` (`account_id`),
  CONSTRAINT `character_account_id` FOREIGN KEY (`account_id`) REFERENCES `Account` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `Guild`
--

DROP TABLE IF EXISTS `Guild`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Guild` (
  `id` int NOT NULL AUTO_INCREMENT,
  `tag` varchar(3) NOT NULL,
  `name` varchar(32) NOT NULL,
  `description` text,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `bank` int NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `GuildRank`
--

DROP TABLE IF EXISTS `GuildRank`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `GuildRank` (
  `id` int NOT NULL AUTO_INCREMENT,
  `guild_id` int NOT NULL,
  `rank` varchar(64) NOT NULL,
  PRIMARY KEY (`id`),
  KEY `guild_rank_guild_id` (`guild_id`),
  CONSTRAINT `guild_rank_guild_id` FOREIGN KEY (`guild_id`) REFERENCES `Guild` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `Inventory`
--

DROP TABLE IF EXISTS `Inventory`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Inventory` (
  `character_id` int NOT NULL,
  `item_id` int NOT NULL,
  `quantity` int NOT NULL,
  PRIMARY KEY (`character_id`,`item_id`),
  CONSTRAINT `inventory_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `Paperdoll`
--

DROP TABLE IF EXISTS `Paperdoll`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Paperdoll` (
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
  CONSTRAINT `paperdoll_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `Position`
--

DROP TABLE IF EXISTS `Position`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Position` (
  `character_id` int NOT NULL,
  `map` int NOT NULL DEFAULT '192',
  `x` int NOT NULL DEFAULT '7',
  `y` int NOT NULL DEFAULT '6',
  `direction` int NOT NULL DEFAULT '2',
  `sitting` int NOT NULL DEFAULT '0',
  `hidden` int NOT NULL DEFAULT '0',
  PRIMARY KEY (`character_id`),
  CONSTRAINT `position_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `Spell`
--

DROP TABLE IF EXISTS `Spell`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Spell` (
  `character_id` int NOT NULL,
  `spell_id` int NOT NULL,
  `level` int NOT NULL DEFAULT '0',
  PRIMARY KEY (`character_id`,`spell_id`),
  CONSTRAINT `spell_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Table structure for table `Stats`
--

DROP TABLE IF EXISTS `Stats`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `Stats` (
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
  CONSTRAINT `stats_character_id` FOREIGN KEY (`character_id`) REFERENCES `Character` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2022-02-11  1:06:02
