-- 0001_init.sql

-- I'm actually using MariaDB which is supposedly a drop-in replacement for MySQL


CREATE TABLE IF NOT EXISTS users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(255) NOT NULL DEFAULT 'player',
    created_timestamp TIMESTAMP NOT NULL DEFAULT UTC_TIMESTAMP,
    email_verified BOOL NOT NULL DEFAULT FALSE,
    has_agreed_terms BOOL NOT NULL DEFAULT FALSE
);


-- refresh tokens to get new JWTs
CREATE TABLE IF NOT EXISTS refresh_tokens (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    client_id VARCHAR(100) NOT NULL, -- NOT the table/row index, but the string client_id
    token VARCHAR(100) NOT NULL UNIQUE,
    created_timestamp TIMESTAMP NOT NULL DEFAULT UTC_TIMESTAMP,
    expires_timestamp TIMESTAMP NOT NULL,
    UNIQUE KEY unique_user_client (user_id, client_id)
);


-- auth codes to send to the client apps, and receive them back
CREATE TABLE IF NOT EXISTS auth_codes (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT NOT NULL,
    client_id VARCHAR(100) NOT NULL,
    code VARCHAR(100) NOT NULL UNIQUE,
    created_timestamp TIMESTAMP NOT NULL DEFAULT UTC_TIMESTAMP,
    expires_timestamp TIMESTAMP NOT NULL
);


CREATE TABLE IF NOT EXISTS client_sites (
    id INT AUTO_INCREMENT PRIMARY KEY,
    client_id VARCHAR(100) NOT NULL UNIQUE, -- public identifier. random string
    hashed_client_secret VARCHAR(255) NOT NULL DEFAULT "", -- only for confidential clients (ie backend, not user)
    name VARCHAR(100) NOT NULL,
    domain VARCHAR(255) NOT NULL UNIQUE,
    redirect_uri VARCHAR(255) NOT NULL,
    description TEXT NOT NULL DEFAULT "",
    logo_url VARCHAR(255) NOT NULL DEFAULT "",
    is_active BOOL NOT NULL DEFAULT TRUE,
    scopes VARCHAR(255) NOT NULL DEFAULT "", -- maybe not needed. Keeping for future-proofing
    client_type VARCHAR(50) NOT NULL DEFAULT "confidential", -- e.g. "confidential" (default), "public", "native" (mobile/desktop)
    category VARCHAR(50) NOT NULL, -- OPTIONS: game, tool, service
    is_internal BOOL NOT NULL DEFAULT FALSE, -- only "TRUE" for self (auth site, this site)
    created_timestamp TIMESTAMP NOT NULL DEFAULT UTC_TIMESTAMP
);


CREATE TABLE dev_blog (
    id INT AUTO_INCREMENT PRIMARY KEY,
    author_name VARCHAR(255) NOT NULL,
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    created_timestamp TIMESTAMP NOT NULL DEFAULT UTC_TIMESTAMP,
    updated_timestamp TIMESTAMP NOT NULL DEFAULT UTC_TIMESTAMP,
    pinned BOOL NOT NULL DEFAULT FALSE
);

-- Codes for email verification, and also for reset password
CREATE TABLE verification_codes (
    user_id INT NOT NULL UNIQUE,
    code_hash VARCHAR(255) NOT NULL, -- ten digits, hashed
    attempts INT NOT NULL DEFAULT 0,
    created_timestamp TIMESTAMP NOT NULL DEFAULT UTC_TIMESTAMP,
    expires_timestamp TIMESTAMP NOT NULL, -- 5 minutes
    FOREIGN KEY (user_id) REFERENCES users(id)
);