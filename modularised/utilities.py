import time
import os

def log_message(message, log_file="app.log"):
    """Log a message to a file."""
    with open(log_file, "a") as file:
        file.write(f"{time.ctime()}: {message}\n")

def fetch_env_variable(var_name):
    """Fetch an environment variable."""
    return os.getenv(var_name)

# More utility functions...
