-- Add migration script here
CREATE TABLE IF NOT EXISTS images (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT UNIQUE NOT NULL,
    url TEXT NOT NULL,
    size INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    timestamp BIGINT NOT NULL,
    extension TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY NOT NULL,
    title TEXT UNIQUE NOT NULL,
    subcats INTEGER NOT NULL,
    files INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS image_categories (
    image_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    PRIMARY KEY(image_id, category_id),
    FOREIGN KEY (image_id) 
        REFERENCES images(id),
    FOREIGN KEY (category_id)
        REFERENCES categories(id)
);

CREATE TABLE IF NOT EXISTS subcategories (
    parent_id INTEGER NOT NULL,
    child_id INTEGER NOT NULL,
    PRIMARY KEY(parent_id, child_id),
    FOREIGN KEY (parent_id) 
        REFERENCES categories(id),
    FOREIGN KEY (child_id)
        REFERENCES categories(id)
);