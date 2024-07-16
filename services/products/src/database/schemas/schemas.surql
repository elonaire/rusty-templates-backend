-- A schema-full user_id table
DEFINE TABLE user_id SCHEMAFULL;
-- define some fields
DEFINE FIELD user_id ON TABLE user_id TYPE string;
DEFINE INDEX userIndex ON TABLE user_id COLUMNS user_id UNIQUE;

-- A schema-full product table
DEFINE TABLE product SCHEMAFULL;
DEFINE FIELD name ON TABLE product TYPE string;
DEFINE FIELD preview_link ON TABLE product TYPE string;
DEFINE FIELD owner ON TABLE product TYPE record<user_id>;


-- A schema-full framework table
DEFINE TABLE framework SCHEMAFULL;
DEFINE FIELD name ON TABLE framework TYPE string
-- Allow only these values in the array
  ASSERT $value INSIDE ["Yew", "Dioxus", "Axum", "Rocket", "Iced", "Tauri", "Actix", "Warp", "Rouille", "Thruster"];

-- A schema-full uses_framework table
DEFINE TABLE uses_framework TYPE RELATION FROM product TO framework SCHEMAFULL PERMISSIONS none;
DEFINE INDEX productFrameworkIndex ON TABLE uses_framework
    COLUMNS in, out UNIQUE;
DEFINE FIELD in ON TABLE uses_framework TYPE record<product>;
DEFINE FIELD out ON TABLE uses_framework TYPE record<framework>;


-- A schema-full application_layer table
DEFINE TABLE application_layer SCHEMAFULL;
DEFINE FIELD name ON TABLE application_layer TYPE string
-- Allow only these values in the array
  ASSERT $value INSIDE ["Frontend", "Backend"];

-- A schema-full belongs_to_application_layer table
DEFINE TABLE belongs_to_application_layer TYPE RELATION FROM product TO application_layer SCHEMAFULL;
DEFINE INDEX applicationLayerIndex ON TABLE belongs_to_application_layer
COLUMNS in, out UNIQUE;
DEFINE FIELD in ON TABLE belongs_to_application_layer TYPE record<product>;
DEFINE FIELD out ON TABLE belongs_to_application_layer TYPE record<application_layer>;

-- A schema-full ui_framework table
DEFINE TABLE ui_framework SCHEMAFULL;
DEFINE FIELD name ON TABLE ui_framework TYPE string
-- Allow only these values in the array
  ASSERT $value INSIDE ["Rusty UI"];

-- A schema-full uses_ui_framework table
DEFINE TABLE uses_ui_framework TYPE RELATION FROM product TO ui_framework SCHEMAFULL;
DEFINE INDEX uiFrameworkIndex ON TABLE uses_ui_framework
COLUMNS in, out UNIQUE;
DEFINE FIELD in ON TABLE uses_ui_framework TYPE record<product>;
DEFINE FIELD out ON TABLE uses_ui_framework TYPE record<ui_framework>;

-- A schema-full use_case table
DEFINE TABLE use_case SCHEMAFULL;
DEFINE FIELD name on TABLE use_case TYPE string
-- Allow only these values in the array
  ASSERT $value INSIDE ["Dashboard", "Ecommerce", "Admin", "Ecommerce Admin", "Finance Admin", "IoT Admin"];

-- A schema-full belongs_to_use_case table
DEFINE TABLE belongs_to_use_case TYPE RELATION FROM product TO use_case SCHEMAFULL;
DEFINE INDEX useCaseIndex ON TABLE belongs_to_use_case
COLUMNS in, out UNIQUE;
DEFINE FIELD in ON TABLE belongs_to_use_case TYPE record<product>;
DEFINE FIELD out ON TABLE belongs_to_use_case TYPE record<use_case>;
