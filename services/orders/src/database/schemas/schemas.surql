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

-- A schema-full order table
DEFINE TABLE order SCHEMAFULL;
DEFINE FIELD status on TABLE order TYPE string
    -- Allow only these values in the array
    ASSERT $value INSIDE ["Pending", "Confirmed", "Ready", "Completed", "Failed", "Refunded", "OnHold"];
DEFINE FIELD created_at ON TABLE order DEFAULT time::now() READONLY;
DEFINE FIELD updated_at ON TABLE order TYPE datetime
    VALUE time::now();
DEFINE FIELD in ON TABLE order TYPE record<user_id>;
DEFINE FIELD out ON TABLE order TYPE record<cart>;

-- A schema-full cart table
DEFINE TABLE cart SCHEMAFULL;
DEFINE FIELD archived ON TABLE cart TYPE bool DEFAULT false;
DEFINE FIELD total_amount ON TABLE cart TYPE int DEFAULT 0;
DEFINE FIELD created_at ON TABLE cart DEFAULT time::now() READONLY;
DEFINE FIELD updated_at ON TABLE cart TYPE datetime
    VALUE time::now();
DEFINE FIELD owner ON TABLE cart TYPE option<record<user_id>>;
DEFINE FIELD session_id ON TABLE cart TYPE string;
DEFINE INDEX sessionIdIndex ON TABLE cart COLUMNS session_id UNIQUE;

-- A schema-full cart table
DEFINE TABLE cart_product SCHEMAFULL;
DEFINE FIELD quantity ON TABLE cart_product TYPE int;
DEFINE FIELD in ON TABLE cart_product TYPE record<cart>;
DEFINE FIELD out ON TABLE cart_product TYPE record<product_id>;
DEFINE INDEX productIndex ON TABLE cart_product COLUMNS in, out UNIQUE;

/* Migrations */
-- Migration for cart table - price column(from float to int)
UPDATE cart SET total_amount = <int>$value;
-- Migration for order table - add new screenshot column
UPDATE cart SET session_id = "" WHERE session_id IS NONE;
