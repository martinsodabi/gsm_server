--atlas schema apply --env turso --to file://src/migrations/000001_down.sql --dev-url "sqlite://dev?mode=memory"
DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS profiles;
