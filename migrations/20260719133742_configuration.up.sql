-- Add up migration script here
CREATE TABLE configuration (
  key character varying NOT NULL,
  value jsonb,
  updated_at timestamp with time zone NOT NULL DEFAULT now (),
  created_at timestamp with time zone NOT NULL DEFAULT now (),
  PRIMARY KEY (key)
);
CREATE INDEX idx_key_values ON configuration USING hash (key);