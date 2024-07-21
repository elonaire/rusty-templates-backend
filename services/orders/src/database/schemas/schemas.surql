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
DEFINE FIELD created_at ON TABLE order TYPE datetime
  DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE order TYPE datetime
    VALUE time::now();
DEFINE FIELD in ON TABLE order TYPE record<user_id>;
DEFINE FIELD out ON TABLE order TYPE record<cart>;

-- A schema-full cart table
DEFINE TABLE cart SCHEMAFULL;
DEFINE FIELD archived ON TABLE cart TYPE bool DEFAULT false;
DEFINE FIELD total_amount ON TABLE cart TYPE float;
DEFINE FIELD updated_at ON TABLE cart TYPE datetime
    VALUE time::now();
DEFINE FIELD owner ON TABLE cart TYPE record<user_id>;

-- A schema-full cart table
DEFINE TABLE cart_product SCHEMAFULL;
DEFINE FIELD quantity ON TABLE cart_product TYPE int;
DEFINE FIELD in ON TABLE cart_product TYPE record<cart>;
DEFINE FIELD out ON TABLE cart_product TYPE record<product_id>;
DEFINE INDEX productIndex ON TABLE cart_product COLUMNS in, out UNIQUE;
