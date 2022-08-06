BEGIN;

  CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

  CREATE TABLE item (
    id uuid NOT NULL,
    name text NOT NULL,
    description text NOT NULL,
    total_amount real NOT NULL,
    current_amount real NOT NULL,
    unit text NOT NULL,
    bought boolean NOT NULL DEFAULT false,
    tags text[] NOT NULL DEFAULT '{}',
    shopping_list_id uuid NOT NULL,
    CONSTRAINT item_pk PRIMARY KEY (id)
  );

  CREATE TABLE shopping_list (
    id uuid NOT NULL,
    title text NOT NULL,
    description text NOT NULL,
    owner_id uuid NOT NULL,
    CONSTRAINT shopping_list_pk PRIMARY KEY (id)
  );

  ALTER TABLE item
  ADD CONSTRAINT item_shopping_list_id_fk FOREIGN KEY (shopping_list_id) REFERENCES shopping_list (id) ON DELETE CASCADE;

  CREATE TABLE users (
    id uuid NOT NULL,
    username text NOT NULL,
    password text NOT NULL,
    CONSTRAINT user_pk PRIMARY KEY (id)
  );
  CREATE UNIQUE INDEX username_idx on users (username ASC);

  ALTER TABLE shopping_list
  ADD CONSTRAINT shopping_list_user_id_fk FOREIGN KEY (owner_id) REFERENCES users (id) ON DELETE CASCADE;

  CREATE TABLE shopping_list_share (
    shopping_list_id uuid NOT NULL,
    target_user_id uuid NOT NULL,
    CONSTRAINT user_to_user PRIMARY KEY (shopping_list_id, target_user_id),
    UNIQUE (shopping_list_id, target_user_id)
  );

  ALTER TABLE shopping_list_share
  ADD CONSTRAINT shared_list_id FOREIGN KEY (shopping_list_id) REFERENCES shopping_list (id) ON DELETE CASCADE;
  ALTER TABLE shopping_list_share
  ADD CONSTRAINT shared_to_user_id FOREIGN KEY (target_user_id) REFERENCES users (id) ON DELETE CASCADE;
      
COMMIT;
