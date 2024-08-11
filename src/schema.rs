// @generated automatically by Diesel CLI.

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
        admin -> Text,
        telegram -> Text,
        twitter -> Nullable<Text>,
        website -> Nullable<Text>,
    }
}

diesel::table! {
    users (id) {
        id -> Int8,
        tgid -> Text,
        credits -> Int8,
    }
}

diesel::table! {
    pumpmen (id) {
        id -> Int8,
        created_at -> Date,
        owner -> Text,
        address -> Text,
        mint -> Text,
        batch -> BigInt,
        tx_fee -> Decimal,
        amount -> Decimal,
        speed -> BigInt,
        bump -> BigInt,
        spent -> Decimal,
        charged -> Decimal,
        deposited -> Numeric
    }
}

diesel::allow_tables_to_appear_in_same_query!(coins, takeovers, users);
