// @generated automatically by Diesel CLI.

diesel::table! {
    bookmarks (id) {
        id -> Nullable<Integer>,
        title -> Text,
        url -> Text,
        store_id -> Nullable<Integer>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    browsing_history (id) {
        id -> Nullable<Integer>,
        tab_id -> Integer,
        url -> Text,
        title -> Nullable<Text>,
        visited_at -> Timestamp,
    }
}

diesel::table! {
    cached_carts (id) {
        id -> Nullable<Integer>,
        store_url -> Text,
        cart_id -> Text,
        cart_data -> Text,
        cached_at -> Timestamp,
    }
}

diesel::table! {
    cached_products (id) {
        id -> Nullable<Integer>,
        store_url -> Text,
        product_id -> Text,
        product_data -> Text,
        cached_at -> Timestamp,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        title -> Text,
        body -> Text,
        published -> Bool,
    }
}

diesel::table! {
    store_directory (id) {
        id -> Nullable<Integer>,
        name -> Text,
        url -> Text,
        description -> Nullable<Text>,
        icon_url -> Nullable<Text>,
        metadata -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    tabs (id) {
        id -> Nullable<Integer>,
        title -> Text,
        url -> Text,
        mode -> Text,
        store_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_credentials (id) {
        id -> Nullable<Integer>,
        store_url -> Text,
        username -> Nullable<Text>,
        encrypted_password -> Nullable<Text>,
        auth_token -> Nullable<Text>,
        token_expires_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_preferences (id) {
        id -> Nullable<Integer>,
        key -> Text,
        value -> Text,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(bookmarks -> store_directory (store_id));
diesel::joinable!(browsing_history -> tabs (tab_id));
diesel::joinable!(tabs -> store_directory (store_id));

diesel::allow_tables_to_appear_in_same_query!(
    
    bookmarks,
    browsing_history,
    cached_carts,
    cached_products,
    
    posts,
    store_directory,
    tabs,
    user_credentials,
    user_preferences,
);
