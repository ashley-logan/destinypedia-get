-- parent_id = 7162, child_id = 8810
-- parent title = Images of Pulse Rifles
-- child title = Images of Outbreak Prime
SELECT * FROM categories WHERE EXISTS (
    SELECT 1
    FROM subcategories as sc
    JOIN categories AS c ON sc.parent_id = c.id
    WHERE categories.id = sc.child_id AND c.title = 'Images of Pulse Rifles');