# database_operations.py

import sqlite3
from datetime import datetime


def initialize_database(db_name='lobsterpot.db'):
    """Initialize the SQLite database with the necessary tables."""
    try:
        conn = sqlite3.connect(db_name)
        cursor = conn.cursor()

        # Create tables
        cursor.execute('''
        CREATE TABLE IF NOT EXISTS interactions (
            id INTEGER PRIMARY KEY,
            timestamp TEXT,
            interaction_details TEXT,
            outcome TEXT
        )''')

        cursor.execute('''
        CREATE TABLE IF NOT EXISTS trapping_methods (
            id INTEGER PRIMARY KEY,
            method_name TEXT,
            description TEXT,
            success_rate REAL
        )''')

        cursor.execute('''
        CREATE TABLE IF NOT EXISTS ai_signatures (
            id INTEGER PRIMARY KEY,
            signature TEXT,
            threat_level TEXT,
            associated_methods TEXT
        )''')

        cursor.execute('''
        CREATE TABLE IF NOT EXISTS file_hashes (
            id INTEGER PRIMARY KEY,
            file_hash TEXT UNIQUE,
            file_path TEXT,
            timestamp TEXT
        )''')

        cursor.execute('''
        CREATE TABLE IF NOT EXISTS quarantine_log (
            id INTEGER PRIMARY KEY,
            file_hash TEXT UNIQUE,
            file_path TEXT,
            quarantine_time TEXT
        )''')

        # Create any other necessary tables here...
        cursor.execute('''CREATE TABLE IF NOT EXISTS file_hashes (
                          id INTEGER PRIMARY KEY,
                          file_hash TEXT UNIQUE,
                          file_path TEXT,
                          timestamp TEXT)''')
    
    finally:
        if conn:
            conn.close()        
        conn.commit()
        print(f"Database {db_name} initialized successfully.")
    except sqlite3.DatabaseError as db_err:
    log_message(f"Database error during initialization: {db_err}")
    print(f"Database error occurred: {db_err}")
    except Exception as e:
    print(f"An unexpected error occurred during database initialization: {e}")
    finally:
    if conn:
            conn.close()

def insert_file_hash(file_hash, file_path, db_name='lobsterpot.db'):
    """Insert file hash into the database."""
    try:
        conn = sqlite3.connect(db_name)
        cursor = conn.cursor()
        timestamp = datetime.now().isoformat()
        cursor.execute('''INSERT OR IGNORE INTO file_hashes (file_hash, file_path, timestamp)
                          VALUES (?, ?, ?)''', (file_hash, file_path, timestamp))
        conn.commit()
    except sqlite3.DatabaseError as db_err:
        log_message(f"Database error while inserting file hash: {db_err}")
    except Exception as e:
        log_message(f"Unexpected error while inserting file hash: {e}")
    finally:
        if conn:
            conn.close()

def log_interaction(interaction_details, outcome, db_name='lobsterpot.db'):
    """Log an interaction into the database."""
    conn = None
    try:
        conn = sqlite3.connect(db_name)
        cursor = conn.cursor()
        timestamp = datetime.now().isoformat()
        cursor.execute('''
            INSERT INTO interactions (timestamp, interaction_details, outcome)
            VALUES (?, ?, ?)''', (timestamp, interaction_details, outcome))
        conn.commit()
        print(f"Logged interaction: {interaction_details} - Outcome: {outcome}")
    except sqlite3.DatabaseError as db_err:
        print(f"Database error occurred: {db_err}")
    except Exception as e:
        print(f"An unexpected error occurred while logging interaction: {e}")
    finally:
        if conn:
            conn.close()
