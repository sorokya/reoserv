CREATE TABLE inventory (
  character_id INT NOT NULL,
  item_id INT NOT NULL,
  quantity INT NOT NULL,
  PRIMARY KEY (character_id, item_id)
);