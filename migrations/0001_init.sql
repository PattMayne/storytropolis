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
    token VARCHAR(100) NOT NULL UNIQUE,
    created_timestamp TIMESTAMP NOT NULL DEFAULT UTC_TIMESTAMP,
    expires_timestamp TIMESTAMP NOT NULL
);



CREATE TABLE blog_post (
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