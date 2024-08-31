# data_processing.py

import sqlite3
from datetime import datetime
from scapy.layers.inet import IP, TCP, UDP, ICMP
from scapy.layers.http import HTTPRequest  # Import for HTTP specific features


def process_packet(packet):
    """Process captured packets."""
    try:
        features = extract_features(packet)
        is_threat = detect_threat(features)
        if is_threat:
            print(f"Threat detected: {packet.summary()}")
            update_firewall(packet)
    except KeyError as e:
        log_message(f"Key error in packet processing: {e}")
    except Exception as e:
        log_message(f"Unexpected error while processing packet: {e}")
        update_firewall(packet)

def connect_to_database(db_name='lobsterpot.db'):
    """Establish a connection to the SQLite database."""
    try:
        conn = sqlite3.connect(db_name)
        return conn
    except sqlite3.Error as e:
        print(f"Database connection error: {e}")
        return None

def initialize_database(db_name='lobsterpot.db'):
    """Initialize the SQLite database by creating necessary tables."""
    conn = connect_to_database(db_name)
    if conn is None:
        return
    cursor = conn.cursor()
    try:
        # Create a table for storing file hashes
        cursor.execute('''CREATE TABLE IF NOT EXISTS file_hashes (
                            id INTEGER PRIMARY KEY,
                            file_hash TEXT UNIQUE,
                            file_path TEXT,
                            timestamp TEXT)''')

       # Create tables for interactions, trapping methods, and AI signatures
       cursor.execute('''CREATE TABLE IF NOT EXISTS interactions (
                       id INTEGER PRIMARY KEY,
                       timestamp TEXT,
                       interaction_details TEXT,
                       outcome TEXT)''')

       cursor.execute('''CREATE TABLE IF NOT EXISTS trapping_methods (
                       id INTEGER PRIMARY KEY,
                       method_name TEXT,
                       description TEXT,
                       success_rate REAL)''')

       cursor.execute('''CREATE TABLE IF NOT EXISTS ai_signatures (
                       id INTEGER PRIMARY KEY,
                       signature TEXT,
                       threat_level TEXT,
                       associated_methods TEXT)''')

       cursor.execute('''CREATE TABLE IF NOT EXISTS file_hashes (
                       id INTEGER PRIMARY KEY,
                       file_hash TEXT UNIQUE,
                       file_path TEXT,
                       timestamp TEXT)''')
 
       # Create a table for known threats
       cursor.execute('''CREATE TABLE IF NOT EXISTS known_threats (
                       id INTEGER PRIMARY KEY,
                       ip TEXT,
                       url TEXT,
                       hash TEXT,
                       description TEXT)''')
   
       # New table for quarantine logs
       cursor.execute('''CREATE TABLE IF NOT EXISTS quarantine_log (
                       id INTEGER PRIMARY KEY,
                       file_hash TEXT UNIQUE,
                       file_path TEXT,
                       quarantine_time TEXT)''')

        # Create more tables as needed...
        
        conn.commit()
    except sqlite3.Error as e:
        print(f"Error creating table: {e}")
    finally:
        conn.close()

def insert_file_hash(file_hash, file_path, db_name='lobsterpot.db'):
    """Insert a file hash and its path into the database."""
    conn = connect_to_database(db_name)
    if conn is None:
        return
    cursor = conn.cursor()
    timestamp = datetime.now().isoformat()
    try:
        cursor.execute('''INSERT OR IGNORE INTO file_hashes (file_hash, file_path, timestamp)
                          VALUES (?, ?, ?)''', (file_hash, file_path, timestamp))
        conn.commit()
        print(f"Inserted file hash {file_hash} for file {file_path}")
    except sqlite3.DatabaseError as db_err:
        print(f"Database error occurred: {db_err}")
    except Exception as e:
        print(f"An unexpected error occurred while inserting file hash: {e}")
    finally:
        conn.close()

def fetch_file_hash(file_hash, db_name='lobsterpot.db'):
    """Fetch details of a file hash from the database."""
    conn = connect_to_database(db_name)
    if conn is None:
        return None
    cursor = conn.cursor()
    try:
        cursor.execute('''SELECT * FROM file_hashes WHERE file_hash = ?''', (file_hash,))
        result = cursor.fetchone()
        return result
    except sqlite3.Error as e:
        print(f"Error fetching file hash: {e}")
        return None
    finally:
        conn.close()

def delete_file_hash(file_hash, db_name='lobsterpot.db'):
    """Delete a file hash from the database."""
    conn = connect_to_database(db_name)
    if conn is None:
        return
    cursor = conn.cursor()
    try:
        cursor.execute('''DELETE FROM file_hashes WHERE file_hash = ?''', (file_hash,))
        conn.commit()
        print(f"Deleted file hash {file_hash}")
    except sqlite3.Error as e:
        print(f"Error deleting file hash: {e}")
    finally:
        conn.close()

def close_database(conn):
    """Close the database connection."""
    if conn:
        conn.close()

# Example usage
if __name__ == "__main__":
    initialize_database()
    # Example operations:
    insert_file_hash('d41d8cd98f00b204e9800998ecf8427e', '/path/to/file')
    print(fetch_file_hash('d41d8cd98f00b204e9800998ecf8427e'))
    delete_file_hash('d41d8cd98f00b204e9800998ecf8427e')
