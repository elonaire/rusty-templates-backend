-- Enable GraphQL
-- DEFINE CONFIG GRAPHQL AUTO;

-- A schema-full user_id table
DEFINE TABLE user_id SCHEMAFULL;
-- define some fields
DEFINE FIELD user_id ON TABLE user_id TYPE string;
DEFINE INDEX userIndex ON TABLE user_id COLUMNS user_id UNIQUE;

-- A schema-full file_id table
DEFINE TABLE file_id SCHEMAFULL;
-- define some fields
DEFINE FIELD file_id ON TABLE file_id TYPE string;
DEFINE INDEX fileIndex ON TABLE file_id COLUMNS file_id UNIQUE;

-- A schema-full product table
DEFINE TABLE product SCHEMAFULL;
DEFINE FIELD name ON TABLE product TYPE string;
DEFINE FIELD slug ON TABLE product VALUE string::slug(name);
-- DEFINE EVENT product_created ON TABLE product WHEN $event = "CREATE" THEN (
--     -- create slug
--     $this.slug = string::slug($after.name)
-- );
DEFINE FIELD price ON TABLE product TYPE int;
DEFINE FIELD preview_link ON TABLE product TYPE string;
DEFINE FIELD details_file ON TABLE product TYPE string;
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


-- A schema-full license table
DEFINE TABLE license SCHEMAFULL;
DEFINE FIELD name ON TABLE license TYPE string
    -- Allow only these values in the array
    ASSERT $value INSIDE ["Rusty", "Rustier", "Rustiest"];
DEFINE FIELD price_factor ON TABLE license TYPE int;
DEFINE FIELD short_description ON TABLE license TYPE string;
DEFINE INDEX licenseIndex ON TABLE license COLUMNS name UNIQUE;
-- Migration for cart_product table - add new license column
CREATE license SET name = "Rusty", price_factor = 1, short_description = "Rusty (1 project sold to 1 client)";
CREATE license SET name = "Rustier", price_factor = 2, short_description = "Rustier (1 project sold to 1 client, DevOps, unlimited personal)";
CREATE license SET name = "Rustiest", price_factor = 9, short_description = "Rustiest (unlimited projects, DevOps, SaaS)";

-- Relationship between product, license and file(artifact)
DEFINE TABLE product_license_artifact SCHEMAFULL TYPE RELATION IN product OUT file_id;
DEFINE FIELD license ON TABLE product_license_artifact TYPE record<license>;
DEFINE INDEX fileIndex ON TABLE product_license_artifact
    COLUMNS in, out, license UNIQUE;
DEFINE FIELD in ON TABLE product_license_artifact TYPE record<product>;
DEFINE FIELD out ON TABLE product_license_artifact TYPE record<file_id>;


/* Migrations */
-- Migration for product table - add new screenshot column
-- UPDATE product SET screenshot = "" WHERE screenshot IS NONE;

-- Migration for product table - price column(from float to int)
-- UPDATE product SET price = <int>$this.price;

-- Migration for product table - slug column
-- UPDATE product SET slug = string::slug($this.name);
-- UPDATE product SET details_file = "" WHERE details_file IS NONE;

-- Migration for product table - ui_framework column
-- UPDATE product SET ui_framework = "RustyUI" WHERE ui_framework="Rusty UI";
