-- Enable GraphQL
-- DEFINE CONFIG GRAPHQL AUTO;

-- A schema-full user_id table
DEFINE TABLE user_id SCHEMAFULL;
-- define some fields
DEFINE FIELD user_id ON TABLE user_id TYPE string;
DEFINE INDEX userIndex ON TABLE user_id COLUMNS user_id UNIQUE;

-- A schema-full product_sku_id table
DEFINE TABLE product_sku_id SCHEMAFULL;
-- define some fields
DEFINE FIELD product_sku_id ON TABLE product_sku_id TYPE string;
DEFINE INDEX productIndex ON TABLE product_sku_id COLUMNS product_sku_id UNIQUE;

-- A schema-full order table
DEFINE TABLE order SCHEMAFULL TYPE RELATION IN user_id OUT cart;
DEFINE FIELD status ON TABLE order TYPE string
    -- Allow only these values in the array
    ASSERT $value INSIDE ["Pending", "Confirmed", "Ready", "Completed", "Failed", "Refunded", "OnHold"];
DEFINE FIELD created_at ON TABLE order TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD updated_at ON TABLE order TYPE datetime
    VALUE time::now();

-- A schema-full cart table
DEFINE TABLE cart SCHEMAFULL TYPE NORMAL;
DEFINE FIELD archived ON TABLE cart TYPE bool DEFAULT false;
-- DEFINE FIELD total_amount ON TABLE cart TYPE int DEFAULT 0;
DEFINE FIELD owner ON TABLE cart TYPE option<record<user_id>>;
DEFINE FIELD session_id ON TABLE cart TYPE string;
DEFINE FIELD created_at ON TABLE cart TYPE datetime DEFAULT time::now() READONLY;
DEFINE FIELD updated_at ON TABLE cart TYPE datetime
    VALUE time::now();

-- DEFINE INDEX sessionIdIndex ON TABLE cart COLUMNS session_id,id,archived UNIQUE;

-- A schema-full cart table
DEFINE TABLE cart_product SCHEMAFULL TYPE RELATION IN product_sku_id OUT cart;
DEFINE FIELD quantity ON TABLE cart_product TYPE int
    ASSERT $value <= 1 AND $value >= 0;
DEFINE INDEX productSkuIndex ON TABLE cart_product COLUMNS in, out UNIQUE;

/* Migrations */
-- Migration for cart table - price column(from float to int)
-- UPDATE cart SET total_amount = <int>$this.total_amount;
-- Migration for order table - add new screenshot column
-- UPDATE cart SET session_id = "" WHERE session_id IS NONE;

-- UPDATE cart_product SET license = (SELECT VALUE id FROM ONLY license WHERE name = "Standard" LIMIT 1) WHERE license IS NONE;
-- UPDATE cart_product SET quantity = 1 WHERE quantity > 1;

-- Migration for product table - add new artifact column
-- UPDATE cart_product SET artifact = "" WHERE artifact IS NONE;

-- Migration for order table
-- REMOVE INDEX IF EXISTS cartOrderIndex ON TABLE order;
