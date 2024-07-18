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

diesel::allow_tables_to_appear_in_same_query!(coins, takeovers, users);
