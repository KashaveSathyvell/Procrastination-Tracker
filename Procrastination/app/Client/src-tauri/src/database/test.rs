#[cfg(test)]
mod tests {
    // Import everything from your sqlite.rs file
    use super::super::sqlite::*;
    use rusqlite::Connection;

    // Helper function to create an in-memory database for testing
    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().expect("Failed to create in-memory DB");
        conn.execute(
            "CREATE TABLE feature_vectors (
                id INTEGER PRIMARY KEY,
                typing_speed REAL,
                idle_ratio REAL,
                truth_label TEXT
            )",
            [],
        ).unwrap();
        conn
    }

    #[test]
    fn test_insert_feature_vector() {
        let conn = setup_test_db();
        let result = conn.execute(
            "INSERT INTO feature_vectors (typing_speed, idle_ratio) VALUES (?1, ?2)",
            [1.2, 0.1],
        );

        assert!(result.is_ok(), "Database should return Ok upon successful insertion");

        let count: i32 = conn.query_row("SELECT COUNT(*) FROM feature_vectors", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1, "There should be exactly 1 row in the database");
    }

    #[test]
    fn test_update_truth_label() {
        let conn = setup_test_db();
        conn.execute("INSERT INTO feature_vectors (typing_speed, idle_ratio) VALUES (1.2, 0.1)", []).unwrap();

        let row_id = 1;
        let new_label = "At Risk";
        let update_result = conn.execute(
            "UPDATE feature_vectors SET truth_label = ?1 WHERE id = ?2",
            [new_label, &row_id.to_string()],
        );

        assert!(update_result.is_ok(), "Update query should succeed");

        let updated_label: String = conn.query_row(
            "SELECT truth_label FROM feature_vectors WHERE id = 1", [], |row| row.get(0)
        ).unwrap();
        assert_eq!(updated_label, "At Risk", "The truth label should now be 'At Risk'");
    }

    #[test]
    fn test_fetch_unlabelled_vectors() {
        let conn = setup_test_db();
        conn.execute("INSERT INTO feature_vectors (typing_speed, truth_label) VALUES (1.5, 'Focused')", []).unwrap();
        conn.execute("INSERT INTO feature_vectors (typing_speed, truth_label) VALUES (0.5, NULL)", []).unwrap();

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM feature_vectors WHERE truth_label IS NULL").unwrap();
        let unlabelled_count: i32 = stmt.query_row([], |row| row.get(0)).unwrap();

        assert_eq!(unlabelled_count, 1, "Should only retrieve the 1 unlabelled row");
    }
}