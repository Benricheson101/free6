table! {
    guilds (id) {
        id -> Int4,
        guild_id -> Int8,
        prefix -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        user_id -> Int8,
        guild_id -> Int8,
        xp -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(guilds, users,);
