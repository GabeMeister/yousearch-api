DROP TABLE IF EXISTS todos;

CREATE TABLE todos (id serial PRIMARY KEY, note TEXT NOT NULL);

DROP TABLE IF EXISTS users;

CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  first_name TEXT,
  last_name TEXT,
  email TEXT,
  password TEXT,
  user_type TEXT
);