-- Your SQL goes here
CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    size INTEGER,
    width INTEGER,
    height INTEGER,
    timestamp TEXT
);

CREATE TABLE IF NOT EXISTS image_categories (
    image_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY (image_id, category_id),
    FOREIGN KEY (image_id) REFERENCES images(id),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    subcats INTEGER,
    files INTEGER
);

CREATE TABLE IF NOT EXISTS subcategories (
    category_id INTEGER NOT NULL,
    subcategory_id INTEGER NOT NULL,
    PRIMARY KEY (category_id, subcategory_id),
    FOREIGN KEY (category_id) REFERENCES categories(id),
    FOREIGN KEY (subcategory_id) REFERENCES categories(id)
);