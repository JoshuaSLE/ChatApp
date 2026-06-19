-- Extensions
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Users table
CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  username VARCHAR(30) UNIQUE NOT NULL,
  password TEXT NOT NULL,
  bio VARCHAR(255),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  last_seen TIMESTAMPTZ,
  online BOOLEAN NOT NULL DEFAULT FALSE,
  avatar_key TEXT
);

-- Refresh Tokens
CREATE TABLE IF NOT EXISTS refresh_tokens (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash TEXT NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Rooms
CREATE TABLE IF NOT EXISTS rooms (
  id UUID PRIMARY KEY,
  name TEXT,
  created_by UUID REFERENCES users(id) ON DELETE SET NULL,
  is_direct BOOLEAN DEFAULT FALSE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Messages
CREATE TABLE IF NOT EXISTS messages (
  id UUID PRIMARY KEY,
  room_id UUID NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
  user_id UUID REFERENCES users(id) ON DELETE SET NULL,
  body TEXT,
  attachment_key TEXT,
  attachment_type TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Room members
CREATE TABLE IF NOT EXISTS room_members (
  user_id UUID REFERENCES users(id) ON DELETE CASCADE,
  room_id UUID REFERENCES rooms(id) ON DELETE CASCADE,
  joined_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
  last_read_message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
  PRIMARY KEY (user_id, room_id)
);

-- Alterations
ALTER TABLE messages
ADD CONSTRAINT check_message_not_empty
CHECK (body IS NOT NULL OR attachment_key IS NOT NULL);

-- Indexes
CREATE INDEX ON messages(room_id, created_at DESC);
CREATE INDEX ON messages(user_id);
CREATE INDEX ON refresh_tokens(id);
CREATE INDEX ON refresh_tokens(user_id);
CREATE INDEX ON room_members(room_id);
CREATE INDEX ON users USING GIN (username gin_trgm_ops);
