BEGIN;
  ALTER TABLE item DROP CONSTRAINT item_shopping_list_id_fk;
  ALTER TABLE shopping_list DROP CONSTRAINT shopping_list_user_id_fk;
  ALTER TABLE shopping_list_share DROP CONSTRAINT shared_list_id;
  ALTER TABLE shopping_list_share DROP CONSTRAINT shared_to_user_id;

  DROP TABLE item;
  DROP TABLE shopping_list;
  DROP TABLE users;
  DROP TABLE shopping_list_share;
COMMIT;