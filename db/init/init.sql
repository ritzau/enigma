-- Create the database and user
CREATE
DATABASE foorum;
CREATE
USER foorum WITH ENCRYPTED PASSWORD 'bar';

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE
foorum TO foorum;

-- Connect to the database to set up schema ownership
\c
foorum postgres

-- Grant schema privileges or transfer ownership
ALTER
SCHEMA public OWNER TO foorum;

-- Optional: Set default search_path for the user
ALTER
ROLE foorum SET search_path = public;
