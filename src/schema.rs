// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int8,
        created_at -> Date,
        tgid -> Int8,
        wallet -> Text
    }
}

diesel::table! {
    coins (id) {
        id -> Int8,
        mint -> Text,
        name -> Text,
        symbol -> Text,
        telegram -> Nullable<Text>,
        website -> Nullable<Text>,
        twitter -> Nullable<Text>,
    }
}

diesel::table! {
    takeovers (id) {
        id -> Int8,
        banner -> Nullable<Text>,
        mint -> Text,
        admin -> BigInt,
        telegram -> Text,
        twitter -> Nullable<Text>,
        website -> Nullable<Text>,
    }
}

diesel::table! {
    pumpmen (id) {
        id -> Int8,
        mint -> Text,
        owner -> Int8,
        wallet -> Nullable<Text>,
        created_at -> Date,
        active -> Bool,
        amount -> Decimal,
        priority_fee -> Decimal,
        batch -> Int4,
        speed -> Int4,
        bumps -> BigInt,
        charged -> Decimal,
    }
}

diesel::table! {
    pumpman_global (id) {
        id -> Int8,
        owner -> Int8,
        amount -> Decimal,
        priority_fee -> Decimal,
        batch -> Int4,
        speed -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(coins, takeovers, users);
