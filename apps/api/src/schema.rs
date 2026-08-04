// @generated manually to match Diesel migrations — regenerate with `diesel print-schema` when tables change.

diesel::table! {
    devices (id) {
        id -> Uuid,
        token_hash -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        last_seen_at -> Timestamptz,
    }
}

diesel::table! {
    reports (id) {
        id -> Uuid,
        device_id -> Uuid,
        phone_e164 -> Text,
        country_code -> Text,
        national_number -> Text,
        sms_content -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(reports -> devices (device_id));

diesel::allow_tables_to_appear_in_same_query!(devices, reports);
