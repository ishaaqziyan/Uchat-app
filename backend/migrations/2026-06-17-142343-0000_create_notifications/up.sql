CREATE TABLE notifications (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    actor_id UUID NOT NULL REFERENCES users(id),
    kind INT2 NOT NULL, -- 1: Follow, 2: Unfollow, 3: Comment, 4: Reaction
    post_id UUID REFERENCES posts(id),
    is_read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
