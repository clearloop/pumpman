diesel::table! {
    coins (address) {
        address -> Text,
        name -> Text,
        symbol -> Text,
        image -> Text,
    }
}

diesel::table! {
    takeovers (id) {
        id -> Int8,
        address -> Text,
        banner -> Nullable<Text>,
        telegram -> Text,
        twitter -> Nullable<Text>,
        website -> Nullable<Text>,
    }
}
