CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- User account tables
CREATE TABLE IF NOT EXISTS users (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE CHECK (LENGTH(username) > 0),
    wallet_address VARCHAR(255) NOT NULL UNIQUE CHECK (LENGTH(wallet_address) > 0),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS badges (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    creator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    badge_address VARCHAR(255) NOT NULL UNIQUE,
    badge_name VARCHAR(255) NOT NULL UNIQUE,
    badge_symbol VARCHAR(255) NOT NULL UNIQUE,
    badge_max_supply INT NOT NULL CHECK (badge_max_supply > 0),
    badge_description TEXT NOT NULL,
    badge_image TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS refresh_tokens (
    id UUID DEFAULT uuid_generate_v4() PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE CHECK (LENGTH(token) > 0),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);
