CREATE TABLE spells (
  character_id INT NOT NULL,
  spell_id INT NOT NULL,
  `level` INT NOT NULL DEFAULT 0,
  PRIMARY KEY (character_id, spell_id)
);