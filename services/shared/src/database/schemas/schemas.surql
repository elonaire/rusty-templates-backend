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
DEFINE TABLE comment SCHEMAFULL;
-- define some fields.
DEFINE FIELD content ON TABLE comment TYPE string;
-- DEFINE FIELD published_date ON TABLE comment VALUE <datetime>$value;
DEFINE FIELD created_at ON TABLE comment
  DEFAULT time::now() READONLY;
DEFINE FIELD updated_at ON TABLE comment TYPE datetime
  VALUE time::now();
DEFINE FIELD in ON TABLE comment TYPE record<user_id>;
DEFINE FIELD out ON TABLE comment TYPE record<product_id>;

-- Relationship between comment and comment
DEFINE TABLE has_reply SCHEMAFULL;
DEFINE INDEX commentCommentIndex ON TABLE has_reply
COLUMNS in, out UNIQUE;
DEFINE FIELD in ON TABLE has_reply TYPE record<comment>;
DEFINE FIELD out ON TABLE has_reply TYPE record<comment>;

-- A schema-full reaction table.
DEFINE TABLE reaction SCHEMAFULL;
-- define some fields.
DEFINE FIELD type ON TABLE reaction TYPE string
-- Allow only these values in the array
  ASSERT $value INSIDE ["Like", "Dislike", "Love", "Haha", "Wow", "Sad", "Angry"];
DEFINE FIELD created_at ON TABLE reaction TYPE datetime
  DEFAULT time::now();
DEFINE INDEX reactionIndex ON TABLE reaction
  COLUMNS in, out UNIQUE;
DEFINE INDEX userReactionIndex ON TABLE reaction
    COLUMNS in UNIQUE;
DEFINE FIELD in ON TABLE reaction TYPE record<user_id>;
DEFINE FIELD out ON TABLE reaction TYPE record<comment>;

-- A schema-full rating table.
DEFINE TABLE rating SCHEMAFULL;
-- Define fields.
DEFINE FIELD rating_value ON TABLE rating TYPE int
  ASSERT $value >= 1 AND $value <= 5; -- Assuming rating values between 1 and 5
DEFINE FIELD created_at ON TABLE rating TYPE datetime
  DEFAULT time::now();
DEFINE FIELD updated_at ON TABLE rating TYPE datetime
  VALUE time::now();
DEFINE FIELD in ON TABLE rating TYPE record<user_id>;
DEFINE FIELD out ON TABLE rating TYPE record<product_id>;
-- Create unique index to ensure a user can rate a product only once
DEFINE INDEX ratingIndex ON TABLE rating
  COLUMNS in, out UNIQUE;

-- A computed view table for average product ratings
DEFINE TABLE average_rating SCHEMAFULL;
-- Define fields
DEFINE FIELD product_id ON TABLE average_rating TYPE record<product_id>;
DEFINE FIELD average_rating_value ON TABLE average_rating TYPE float;
DEFINE FIELD rating_count ON TABLE average_rating TYPE int;
  -- Query to compute the average rating
DEFINE TABLE average_rating AS
    SELECT
      out AS product_id,
      math::mean(rating_value) AS average_rating_value,
      count() AS rating_count
    FROM rating
    GROUP BY out;
