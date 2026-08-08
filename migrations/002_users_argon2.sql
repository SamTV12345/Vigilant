-- Vigilant: Add must_change_password column + update default admin to argon2
-- SQLite: ALTER TABLE ADD COLUMN only supports adding columns
ALTER TABLE users ADD COLUMN must_change_password INTEGER NOT NULL DEFAULT 1;

-- Replace the default admin user with an argon2 hash of "admin"
-- Delete old bcrypt admin first, then insert new
DELETE FROM users WHERE id = 'default' AND username = 'admin';

INSERT OR IGNORE INTO users (id, username, password_hash, must_change_password) VALUES
    ('default', 'admin', '$argon2id$v=19$m=19456,t=2,p=1$H7UEAk8yZhX13KWo6Qq85g$Pc+JFmMhuPIfQl7fW/N7U0//mbts1merECbY3CEFjFY', 1);
