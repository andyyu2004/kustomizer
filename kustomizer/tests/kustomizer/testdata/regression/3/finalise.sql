-- Make postgres user usable for debezium replication
ALTER ROLE postgres REPLICATION;

-- Make postgres user able to run migrations on tables owned by epcowner
GRANT epcowner TO postgres;

-- Create the dataset_version table for tracking dataset versions
CREATE TABLE IF NOT EXISTS partly.dataset_version (id SERIAL PRIMARY KEY, version TEXT, timestamp TEXT);