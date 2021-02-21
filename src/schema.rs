table! {
    accounts (id) {
        id -> Integer,
        name -> Varchar,
        password_hash -> Char,
        real_name -> Varchar,
        location -> Varchar,
        email -> Varchar,
        register_ip -> Varchar,
        created_on -> Datetime,
    }
}

table! {
    bank_inventory (character_id, item_id) {
        character_id -> Integer,
        item_id -> Integer,
        quantity -> Integer,
    }
}

table! {
    characters (id) {
        id -> Integer,
        account_id -> Integer,
        name -> Varchar,
        title -> Nullable<Varchar>,
        home -> Nullable<Varchar>,
        fiance -> Nullable<Varchar>,
        partner -> Nullable<Varchar>,
        admin -> Integer,
        class -> Integer,
        gender -> Integer,
        race -> Integer,
        hair_style -> Integer,
        hair_color -> Integer,
        bank_max -> Integer,
        gold_bank -> Integer,
        guild_id -> Nullable<Integer>,
        guild_rank_id -> Nullable<Integer>,
        guild_rank_string -> Nullable<Varchar>,
    }
}

table! {
    guilds (id) {
        id -> Integer,
        tag -> Varchar,
        name -> Varchar,
        description -> Nullable<Text>,
        created_on -> Datetime,
        bank -> Integer,
    }
}

table! {
    guild_ranks (id) {
        id -> Integer,
        guild_id -> Integer,
        rank -> Varchar,
    }
}

table! {
    inventory (character_id, item_id) {
        character_id -> Integer,
        item_id -> Integer,
        quantity -> Integer,
    }
}

table! {
    paperdolls (character_id) {
        character_id -> Integer,
        boots -> Integer,
        accessory -> Integer,
        gloves -> Integer,
        belt -> Integer,
        armor -> Integer,
        necklace -> Integer,
        hat -> Integer,
        shield -> Integer,
        weapon -> Integer,
        ring -> Integer,
        ring2 -> Integer,
        armlet -> Integer,
        armlet2 -> Integer,
        bracer -> Integer,
        bracer2 -> Integer,
    }
}

table! {
    positions (character_id) {
        character_id -> Integer,
        map -> Integer,
        x -> Integer,
        y -> Integer,
        direction -> Integer,
        sitting -> Integer,
        hidden -> Integer,
    }
}

table! {
    spells (character_id, spell_id) {
        character_id -> Integer,
        spell_id -> Integer,
        level -> Integer,
    }
}

table! {
    stats (character_id) {
        character_id -> Integer,
        level -> Integer,
        experience -> Integer,
        hp -> Integer,
        tp -> Integer,
        strength -> Integer,
        intelligence -> Integer,
        wisdom -> Integer,
        agility -> Integer,
        constitution -> Integer,
        charisma -> Integer,
        stat_points -> Integer,
        skill_points -> Integer,
        karma -> Integer,
        usage -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    accounts,
    bank_inventory,
    characters,
    guilds,
    guild_ranks,
    inventory,
    paperdolls,
    positions,
    spells,
    stats,
);
