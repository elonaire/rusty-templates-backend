-- A schema-full user_id table
DEFINE TABLE user_id SCHEMAFULL;
-- define some fields
DEFINE FIELD user_id ON TABLE user_id TYPE string;
DEFINE INDEX userIndex ON TABLE user_id COLUMNS user_id UNIQUE;

-- A schema-full product table
DEFINE TABLE product SCHEMAFULL;
DEFINE FIELD name ON TABLE product TYPE string;
DEFINE FIELD slug ON TABLE product TYPE option<string>;
DEFINE EVENT product_created ON TABLE product WHEN $event = "CREATE" THEN (
    -- create slug
    $value.slug = string::slug($after.name)
);
DEFINE FIELD price ON TABLE product TYPE int;
DEFINE FIELD preview_link ON TABLE product TYPE string;
DEFINE FIELD screenshot ON TABLE product TYPE string;
DEFINE FIELD owner ON TABLE product TYPE record<user_id>;
DEFINE FIELD framework ON TABLE product TYPE string
    -- Allow only these values in the array
  ASSERT $value INSIDE ["Yew", "Dioxus", "Axum", "Rocket", "Iced", "Tauri", "Actix", "Warp", "Rouille", "Thruster"];
DEFINE FIELD application_layer ON TABLE product TYPE string
    -- Allow only these values in the array
  ASSERT $value INSIDE ["Frontend", "Backend"];
DEFINE FIELD ui_framework ON TABLE product TYPE option<string>
    -- Allow only these values in the array
    ASSERT $value INSIDE ["RustyUI", None];
DEFINE FIELD use_case on TABLE product TYPE string
    -- Allow only these values in the array
    ASSERT $value INSIDE ["Dashboard", "Ecommerce", "Admin", "EcommerceAdmin", "FinanceAdmin", "IoTAdmin"];


/* Migrations */
-- Migration for product table - add new screenshot column
UPDATE product SET screenshot = "" WHERE screenshot IS NONE;

-- Migration for product table - price column(from float to int)
UPDATE product SET price = <int>$this.price;

-- Migration for product table - slug column
UPDATE product SET slug = string::slug($this.name);

-- Migration for product table - ui_framework column
UPDATE product SET ui_framework = "RustyUI" WHERE ui_framework="Rusty UI";
