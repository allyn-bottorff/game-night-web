-- Add migrations for the game night web application database schema

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create polls table
CREATE TABLE IF NOT EXISTS polls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    creator_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    FOREIGN KEY (creator_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create options table
CREATE TABLE IF NOT EXISTS options (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    poll_id INTEGER NOT NULL,
    text TEXT NOT NULL,
    is_date BOOLEAN NOT NULL DEFAULT 0,
    date_time TIMESTAMP,
    FOREIGN KEY (poll_id) REFERENCES polls(id) ON DELETE CASCADE
);

-- Create votes table
CREATE TABLE IF NOT EXISTS votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    option_id INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (option_id) REFERENCES options(id) ON DELETE CASCADE,
    UNIQUE(user_id, option_id)
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_polls_creator ON polls(creator_id);
CREATE INDEX IF NOT EXISTS idx_polls_expires ON polls(expires_at);
CREATE INDEX IF NOT EXISTS idx_options_poll ON options(poll_id);
CREATE INDEX IF NOT EXISTS idx_votes_user ON votes(user_id);
CREATE INDEX IF NOT EXISTS idx_votes_option ON votes(option_id);

-- Insert initial admin user (password is 'admin')
INSERT INTO users (username, password_hash, is_admin) 
VALUES ('admin', '$2b$12$BX0YV17r4MKqT2nCuPLCOOh76CN8uBALR2z0xnUu1CU.Rj.p5Oj0a', 1)
ON CONFLICT(username) DO NOTHING;