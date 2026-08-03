-- scam reports submitted by devices (classification columns land in a later issue)
CREATE TABLE IF NOT EXISTS reports (
    id UUID PRIMARY KEY,
    device_id UUID NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    phone_e164 TEXT NOT NULL,
    country_code TEXT NOT NULL,
    national_number TEXT NOT NULL,
    sms_content TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS reports_device_id_created_at_idx
    ON reports (device_id, created_at DESC);

CREATE INDEX IF NOT EXISTS reports_phone_e164_idx
    ON reports (phone_e164);
