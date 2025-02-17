CREATE TABLE IF NOT EXISTS mcaptcha_users (
	name VARCHAR(100) NOT NULL UNIQUE,
	email VARCHAR(100) UNIQUE DEFAULT NULL,
	email_verified BOOLEAN DEFAULT NULL,
    secret varchar(50) NOT NULL UNIQUE,
	password TEXT NOT NULL,
	ID INT auto_increment,
	PRIMARY KEY(ID)
);
