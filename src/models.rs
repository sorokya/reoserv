use chrono::prelude::*;

#[derive(Queryable)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub password_hash: String,
    pub real_name: String,
    pub location: String,
    pub email: String,
    pub computer: String,
    pub hdid: i32,
    pub register_ip: String,
    pub created_on: NaiveDateTime,
}

#[derive(Queryable)]
pub struct Character {
    pub id: i32,
    pub account_id: i32,
    pub name: String,
    pub title: String,
    pub home: String,
    pub fiance: String,
    pub partner: String,
    pub admin: i32,
    pub class: i32,
    pub gender: i32,
    pub race: i32,
    pub hair_style: i32,
    pub hair_color: i32,
    pub bank_max: i32,
    pub gold_bank: i32,
    pub guild_id: i32,
    pub guild_rank_id: i32,
    pub guild_rank_string: String,
}

#[derive(Queryable)]
pub struct Stats {
    pub character_id: i32,
    pub level: i32,
    pub experience: i32,
    pub hp: i32,
    pub tp: i32,
    pub strength: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub agility: i32,
    pub constitution: i32,
    pub charisma: i32,
    pub stat_points: i32,
    pub skill_points: i32,
    pub karma: i32,
    pub usage: i32,
}

#[derive(Queryable)]
pub struct Position {
    pub character_id: i32,
    pub map: i32,
    pub x: i32,
    pub y: i32,
    pub direction: i32,
    pub sitting: i32,
    pub hidden: i32,
}

#[derive(Queryable)]
pub struct Paperdoll {
    pub character_id: i32,
    pub boots: i32,
    pub accessory: i32,
    pub gloves: i32,
    pub belt: i32,
    pub armor: i32,
    pub necklace: i32,
    pub hat: i32,
    pub shield: i32,
    pub weapon: i32,
    pub ring: i32,
    pub ring2: i32,
    pub armlet: i32,
    pub armlet2: i32,
    pub bracer: i32,
    pub bracer2: i32,
}

#[derive(Queryable)]
pub struct Inventory {
    pub character_id: i32,
    pub item_id: i32,
    pub quantity: i32,
}

#[derive(Queryable)]
pub struct BankInventory {
    pub character_id: i32,
    pub item_id: i32,
    pub quantity: i32,
}

#[derive(Queryable)]
pub struct Guild {
    pub id: i32,
    pub tag: String,
    pub name: String,
    pub description: String,
    pub created_on: DateTime<Utc>,
    pub bank: i32,
}

#[derive(Queryable)]
pub struct GuildRank {
    pub id: i32,
    pub guild_id: i32,
    pub rank: String,
}

#[derive(Queryable)]
pub struct Spell {
    pub character_id: i32,
    pub spell_id: i32,
    pub level: i32,
}
