# logging_analysis.py

import sqlite3
import pandas as pd
from datetime import datetime
import logging


# Initialize logging
logging.basicConfig(level=logging.INFO)

def log_interaction(interaction_details, outcome, db_name='lobsterpot.db'):
    """Log an interaction into the database."""
    try:
        conn = sqlite3.connect(db_name)
        cursor = conn.cursor()
        timestamp = datetime.now().isoformat()
        cursor.execute('''INSERT INTO interactions (timestamp, interaction_details, outcome)
                          VALUES (?, ?, ?)''', (timestamp, interaction_details, outcome))
        conn.commit()
    except sqlite3.DatabaseError as db_err:
        logging.error(f"Database error during interaction logging: {db_err}")
    except Exception as e:
        logging.error(f"Unexpected error while logging interaction: {e}")
    finally:
        if conn:
            conn.close()

def analyze_past_interactions(db_name='lobsterpot.db'):
    """Analyze past interactions."""
    try:
        interactions = get_interactions(db_name)
        logging.info("Analyzing past interactions:")
        logging.info(interactions.describe())
    except Exception as e:
        logging.error(f"An error occurred while analyzing past interactions: {e}")

def get_interactions(db_name='lobsterpot.db'):
    """Retrieve interactions from the database."""
    try:
        conn = sqlite3.connect(db_name)
        query = '''SELECT * FROM interactions'''
        df = pd.read_sql(query, conn)
        return df
    except sqlite3.DatabaseError as db_err:
        logging.error(f"Database error occurred: {db_err}")
        return pd.DataFrame()  # Return an empty DataFrame on error
    finally:
        if conn:
            conn.close()

def parse_log(file_path):
    """Parse a log file to extract URLs and analyze them."""
    try:
        with open(file_path, 'r') as file:
            for line in file:
                if "http://" in line or "https://" in line:
                    url = extract_url_from_log_line(line)
                    logging.info(f"URL found in log: {url}")
                    analyze_url(url)  # Assuming analyze_url is defined elsewhere
    except FileNotFoundError:
        logging.error(f"Log file not found: {file_path}")
    except Exception as e:
        logging.error(f"An error occurred while parsing log file: {e}")

def extract_url_from_log_line(line):
    """Extracts a URL from a log line."""
    url_start = line.find("http://") if "http://" in line else line.find("https://")
    url_end = line.find(" ", url_start)
    return line[url_start:url_end]

def log_url_block(url):
    """Log the blocked URL for auditing purposes."""
    try:
        with open("url_blocks.log", "a") as log_file:
            log_file.write(f"{time.ctime()}: Blocked URL {url}\n")
    except IOError as e:
        logging.error(f"Failed to write to log file: {e}")
    except Exception as e:
        logging.error(f"An unexpected error occurred while logging URL block: {e}")

def fetch_threat_data_from_otx(url_to_analyze):
    """Fetch threat data from the OTX API."""
    try:
        pulse_info = otx.get_indicator_details_full(otx.IndicatorTypes.URL, url_to_analyze)  # Assuming otx is initialized elsewhere
        threat_info = {
            "url": url_to_analyze,
            "pulse_info": pulse_info
        }
        return threat_info
    except Exception as e:
        logging.error(f"An error occurred while fetching threat data: {e}")
        return None

# Ensure to remove any duplicated or conflicting function definitions

# Example usage
if __name__ == "__main__":
    log_file_path = "/path/to/your/logfile.log"
    parse_log(log_file_path)
    