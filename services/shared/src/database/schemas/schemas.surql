-- Enable GraphQL
-- DEFINE CONFIG GRAPHQL AUTO;

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

-- A schema-full comment table.
DEFINE TABLE comment SCHEMAFULL TYPE NORMAL;
-- define some fields.
DEFINE FIELD content ON TABLE comment TYPE string;
-- DEFINE FIELD published_date ON TABLE comment VALUE <datetime>$value;
DEFINE FIELD created_at ON TABLE comment TYPE datetime
  DEFAULT time::now() READONLY;
DEFINE FIELD updated_at ON TABLE comment TYPE datetime
  VALUE time::now();

-- A schema-full user_id-comment as writer relation table.
DEFINE TABLE wrote SCHEMAFULL TYPE RELATION IN user_id OUT comment;

-- Relationship between comment and comment
DEFINE TABLE has_reply SCHEMAFULL TYPE RELATION IN comment OUT comment;
DEFINE INDEX commentCommentIndex ON TABLE has_reply
COLUMNS in, out UNIQUE;

-- A schema-full reaction table.
DEFINE TABLE reaction SCHEMAFULL TYPE RELATION IN user_id OUT comment;
-- define some fields.
DEFINE FIELD type ON TABLE reaction TYPE string
-- Allow only these values in the array
  ASSERT $value INSIDE ["Like", "Dislike", "Love", "Haha", "Wow", "Sad", "Angry"];
DEFINE FIELD created_at ON TABLE reaction TYPE datetime
  DEFAULT time::now() READONLY;
DEFINE INDEX reactionIndex ON TABLE reaction
  COLUMNS in, out UNIQUE;

-- A schema-full review table.
DEFINE TABLE review SCHEMAFULL TYPE RELATION IN user_id OUT product_id;
-- Define fields.
DEFINE FIELD rating ON TABLE review TYPE int
  ASSERT $value >= 1 AND $value <= 5; -- Assuming rating values between 1 and 5
DEFINE FIELD created_at ON TABLE review TYPE datetime
  DEFAULT time::now() READONLY;
DEFINE FIELD updated_at ON TABLE review TYPE datetime
  VALUE time::now();
-- Create unique index to ensure a user can rate a product only once
DEFINE INDEX reviewIndex ON TABLE review
  COLUMNS in, out UNIQUE;

-- A schema-full product_id/review-comment relation table.
DEFINE TABLE has_comment SCHEMAFULL TYPE RELATION IN product_id | review OUT comment;

-- A computed view table for average product ratings
DEFINE TABLE average_rating AS
    SELECT
    ->product_id[0].product_id AS product_id,
    math::mean(<float> rating) AS average_rating_value,
    count() AS no_of_reviews
    FROM review
    GROUP BY product_id;
