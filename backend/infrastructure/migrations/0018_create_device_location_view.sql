CREATE VIEW IF NOT EXISTS v_device_location_counts AS
SELECT
    d.device_id,
    d.device_name,
    COUNT(rl.id) AS location_count
FROM devices d
LEFT JOIN resource_locations rl ON rl.device_id = d.device_id
WHERE d.id = (
    SELECT d2.id
    FROM devices d2
    WHERE d2.device_id = d.device_id
    ORDER BY d2.linked_at DESC
    LIMIT 1
)
GROUP BY d.device_id, d.device_name;
