-- Create the database and user
CREATE DATABASE enigma;
CREATE USER enigma WITH ENCRYPTED PASSWORD 'bar';

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE enigma TO enigma;

-- Connect to the database to set up schema ownership
\c enigma postgres

-- Grant schema privileges or transfer ownership
ALTER SCHEMA public OWNER TO enigma;

-- Optional: Set default search_path for the user
ALTER ROLE enigma SET search_path = public;
