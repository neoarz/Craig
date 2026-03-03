CREATE TABLE chat_logs (
    id          BIGSERIAL   PRIMARY KEY,
    user_id     BIGINT      NOT NULL,
    username    TEXT        NOT NULL,
    guild_id    BIGINT,
    channel_id  BIGINT      NOT NULL,
    source      TEXT        NOT NULL,
    model       TEXT        NOT NULL,
    prompt      TEXT        NOT NULL,
    response    TEXT        NOT NULL,
    latency_ms  INT         NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_chat_logs_user_id ON chat_logs (user_id);
CREATE INDEX idx_chat_logs_created_at ON chat_logs (created_at);
