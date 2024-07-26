-- A schema-full user_id table
DEFINE TABLE user_id SCHEMAFULL;
-- define some fields
DEFINE FIELD user_id ON TABLE user_id TYPE string;
DEFINE INDEX userIndex ON TABLE user_id COLUMNS user_id UNIQUE;

-- A schema-full product_id table
DEFINE TABLE product_id SCHEMAFULL;
-- define some fields
DEFINE FIELD product_id ON TABLE product_id TYPE string;
DEFINE INDEX productIndex ON TABLE product_id COLUMNS product_id UNIQUE;

-- A schema-full file table
DEFINE TABLE file SCHEMAFULL;
DEFINE FIELD name ON TABLE file TYPE string;
DEFINE FIELD system_filename ON TABLE file TYPE string;
DEFINE FIELD size ON TABLE file TYPE number;
DEFINE FIELD mime_type ON TABLE file TYPE string;
DEFINE FIELD owner ON TABLE file TYPE record<user_id>;
DEFINE FIELD created_at ON TABLE file TYPE datetime
  DEFAULT time::now();

-- Relationship between product and file
DEFINE TABLE has_file SCHEMAFULL;
DEFINE INDEX fileIndex ON TABLE has_file
COLUMNS in, out UNIQUE;
DEFINE FIELD in ON TABLE has_file TYPE record<product_id>;
DEFINE FIELD out ON TABLE has_file TYPE record<file>;

/* Migrations */
-- Migration for file table
UPDATE file SET system_filename = "" WHERE system_filename IS NONE;
