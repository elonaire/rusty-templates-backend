-- A schema-full user_id table
DEFINE TABLE user_id SCHEMAFULL;
-- define some fields
DEFINE FIELD user_id ON TABLE user_id TYPE string;
DEFINE INDEX userIndex ON TABLE user_id COLUMNS user_id UNIQUE;

-- A schema-full product table
DEFINE TABLE product SCHEMAFULL;
DEFINE FIELD name ON TABLE product TYPE string;
DEFINE FIELD price ON TABLE product TYPE float VALUE <float>$value;
DEFINE FIELD preview_link ON TABLE product TYPE string;
DEFINE FIELD owner ON TABLE product TYPE record<user_id>;
DEFINE FIELD framework ON TABLE product TYPE string
    -- Allow only these values in the array
  ASSERT $value INSIDE ["Yew", "Dioxus", "Axum", "Rocket", "Iced", "Tauri", "Actix", "Warp", "Rouille", "Thruster"];
DEFINE FIELD application_layer ON TABLE product TYPE string
    -- Allow only these values in the array
  ASSERT $value INSIDE ["Frontend", "Backend"];
DEFINE FIELD ui_framework ON TABLE product TYPE option<string>
    -- Allow only these values in the array
    ASSERT $value INSIDE ["Rusty UI", None];
DEFINE FIELD use_case on TABLE product TYPE string
    -- Allow only these values in the array
    ASSERT $value INSIDE ["Dashboard", "Ecommerce", "Admin", "Ecommerce Admin", "Finance Admin", "IoT Admin"];
