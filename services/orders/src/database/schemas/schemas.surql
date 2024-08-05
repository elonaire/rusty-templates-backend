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
DEFINE INDEX cartOrderIndex ON TABLE order COLUMNS in,out UNIQUE;

-- A schema-full cart table
DEFINE TABLE cart SCHEMAFULL;
DEFINE FIELD archived ON TABLE cart TYPE bool DEFAULT false;
DEFINE FIELD total_amount ON TABLE cart TYPE int DEFAULT 0;
DEFINE FIELD created_at ON TABLE cart DEFAULT time::now() READONLY;
DEFINE FIELD updated_at ON TABLE cart TYPE datetime
    VALUE time::now();
DEFINE FIELD owner ON TABLE cart TYPE option<record<user_id>>;
DEFINE FIELD session_id ON TABLE cart TYPE string;
-- DEFINE INDEX sessionIdIndex ON TABLE cart COLUMNS session_id,id,archived UNIQUE;

-- A schema-full cart table
DEFINE TABLE cart_product SCHEMAFULL;
DEFINE FIELD quantity ON TABLE cart_product TYPE int
    ASSERT $value <= 1;
DEFINE FIELD license ON TABLE cart_product TYPE record<license>;
DEFINE FIELD artifact ON TABLE cart_product TYPE string;
DEFINE FIELD in ON TABLE cart_product TYPE record<cart>;
DEFINE FIELD out ON TABLE cart_product TYPE record<product_id>;
DEFINE INDEX productIndex ON TABLE cart_product COLUMNS in, out UNIQUE;

-- A schema-full license table
DEFINE TABLE license SCHEMAFULL;
DEFINE FIELD name ON TABLE license TYPE string
    -- Allow only these values in the array
    ASSERT $value INSIDE ["Rusty", "Rustier", "Rustiest"];
DEFINE FIELD price_factor ON TABLE license TYPE int;
DEFINE FIELD short_description ON TABLE license TYPE string;
DEFINE INDEX licenseIndex ON TABLE license COLUMNS name UNIQUE;

/* Migrations */
-- Migration for cart table - price column(from float to int)
UPDATE cart SET total_amount = <int>$this.total_amount;
-- Migration for order table - add new screenshot column
UPDATE cart SET session_id = "" WHERE session_id IS NONE;

-- Migration for cart_product table - add new license column
CREATE license SET name = "Rusty", price_factor = 1, short_description = "Rusty (1 project sold to 1 client)";
CREATE license SET name = "Rustier", price_factor = 2, short_description = "Rustier (1 project sold to 1 client, DevOps, unlimited personal)";
CREATE license SET name = "Rustiest", price_factor = 9, short_description = "Rustiest (unlimited projects, DevOps, SaaS)";

UPDATE cart_product SET license = (SELECT VALUE id FROM ONLY license WHERE name = "Standard" LIMIT 1) WHERE license IS NONE;
UPDATE cart_product SET quantity = 1 WHERE quantity > 1;

-- Migration for product table - add new artifact column
UPDATE cart_product SET artifact = "" WHERE artifact IS NONE;
