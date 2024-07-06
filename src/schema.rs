diesel::table! {
    coins (mint) {
        description -> Nullable<Text>,
        image -> Nullable<Text>,
        mint -> Text,
        name -> Text,
        symbol -> Text,
        telegram -> Nullable<Text>,
        twitter -> Nullable<Text>,
        website -> Nullable<Text>,
        created_on -> Nullable<Text>,
        dex -> Nullable<Text>
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
