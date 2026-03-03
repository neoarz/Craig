ALTER TABLE chat_logs
    ADD COLUMN IF NOT EXISTS success BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE chat_logs
    ADD COLUMN IF NOT EXISTS error_text TEXT;

CREATE INDEX IF NOT EXISTS idx_chat_logs_success_created_at
    ON chat_logs (success, created_at DESC);
