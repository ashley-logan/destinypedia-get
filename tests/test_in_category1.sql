-- image_id = 3368, category_id = 366
-- images.title = Vault of Glass.jpg
-- categories.title = Fair use images
-- pixels = 51 x 586 = 2500 + 430 + 586 = 3416
-- minpixels = 3000
SELECT * FROM images
WHERE width *  height >= 3000
AND EXISTS (
    SELECT 1
    FROM image_categories AS ic
    JOIN categories ON categories.id = ic.category_id
    WHERE categories.title = 'Fair use images'
    AND ic.image_id = images.id
)
AND extension IN ('PNG', 'JPG');