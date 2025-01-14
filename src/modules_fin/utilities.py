# utilities.py

import time
import os
import json
import hashlib
import subprocess
import logging
from kafka import KafkaProducer


def get_kafka_producer(bootstrap_servers='localhost:9092'):
    """Initialize Kafka producer."""
    return KafkaProducer(
        bootstrap_servers=bootstrap_servers,
        value_serializer=lambda v: json.dumps(v).encode('utf-8')
    )

def send_to_kafka(producer, topic, log_entry):
    """Send a message to a Kafka topic."""
    producer.send(topic, log_entry)

def log_message(message, log_file="app.log"):
    """Log a message to a file."""
    try:
        with open(log_file, "a") as file:
            file.write(f"{time.ctime()}: {message}\n")
    except IOError as e:
        print(f"Failed to write to log file: {e}")

def fetch_env_variable(var_name, default=None):
    """Fetch an environment variable or return a default value."""
    value = os.getenv(var_name, default)
    if value is None:
        log_message(f"Environment variable {var_name} is not set.")
    return value

def set_env_variable(var_name, value):
    """Set an environment variable."""
    os.environ[var_name] = value

def check_file_exists(file_path):
    """Check if a file exists."""
    return os.path.isfile(file_path)

def create_directory(dir_path):
    """Create a directory if it doesn't exist."""
    if not os.path.exists(dir_path):
        os.makedirs(dir_path)

def delete_file(file_path):
    """Delete a file if it exists."""
    if check_file_exists(file_path):
        os.remove(file_path)

def generate_sha256_hash(text):
    """Generate a SHA-256 hash of the given text."""
    return hashlib.sha256(text.encode()).hexdigest()

def run_shell_command(command):
    """Run a shell command and return the output."""
    try:
        output = subprocess.check_output(command, shell=True, stderr=subprocess.STDOUT)
        return output.decode()
    except subprocess.CalledProcessError as e:
        log_message(f"Error running command '{command}': {e.output.decode()}", level="ERROR")
        return None

def read_json(file_path):
    """Read and return JSON data from a file."""
    try:
        with open(file_path, 'r') as file:
            return json.load(file)
    except (FileNotFoundError, json.JSONDecodeError) as e:
        log_message(f"Error reading JSON from {file_path}: {e}", level="ERROR")
        return None

def write_json(data, file_path):
    """Write JSON data to a file."""
    try:
        with open(file_path, 'w') as file:
            json.dump(data, file, indent=4)
    except Exception as e:
        log_message(f"Error writing JSON to {file_path}: {e}", level="ERROR")

def current_timestamp():
    """Return the current timestamp."""
    return time.time()

def retry_on_exception(func, retries=3, delay=2):
    """Retry a function call on exception."""
    def wrapper(*args, **kwargs):
        for attempt in range(retries):
            try:
                return func(*args, **kwargs)
            except Exception as e:
                log_message(f"Error: {e}. Retrying {attempt + 1}/{retries} after {delay}s...", level="WARNING")
                time.sleep(delay)
        return None
    return wrapper

@retry_on_exception
def risky_operation():
    """A function that might fail and needs retrying."""
    # Implementation of a risky operation that might fail
    pass
