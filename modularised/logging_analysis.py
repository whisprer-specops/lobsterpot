import sqlite3
import pandas as pd

def log_interaction(interaction_details, outcome, db_name='lobsterpot.db'):
    """Log an interaction into the database."""
    conn = sqlite3.connect(db_name)
    cursor = conn.cursor()
    timestamp = datetime.now().isoformat()
    cursor.execute('''INSERT INTO interactions (timestamp, interaction_details, outcome)
                      VALUES (?, ?, ?)''', (timestamp, interaction_details, outcome))
    conn.commit()
    conn.close()

def analyze_past_interactions(db_name='lobsterpot.db'):
    """Analyze past interactions."""
    interactions = get_interactions(db_name)
    print("Analyzing past interactions:")
    print(interactions.describe())

# More logging and analysis functions...



def detect_threat(features):
    # Convert features to a format suitable for the AI model
    # Here, you should select the most relevant features for your model
    input_data = np.array([[features.get('packet_length', 0), features.get('proto', 0), features.get('tcp_flags', 0)]])
    
    # Predict using the trained AI model (assumed to be the Central Ganglion)
    prediction = ganglion.predict(input_data)
    is_threat = prediction[0][0] > 0.75  # Example threshold
    return is_threat        

# Start sniffing network traffic
sniff(prn=process_packet, store=False)

# Define setup for parsing IP/URL data from packets and logs
def process_packet(packet):
    if packet.haslayer(IP):
        ip_src = packet[IP].src
        ip_dst = packet[IP].dst
        print(f"Source IP: {ip_src}, Destination IP: {ip_dst}")

    if packet.haslayer(HTTPRequest):
        url = packet[HTTPRequest].Host.decode() + packet[HTTPRequest].Path.decode()
        print(f"URL: {url_analysis}")

        # Here you could send the URL to VirusTotal or another API
        analyze_url(analysis_url)

sniff(prn=process_packet, store=False)

# Example of log file parsing (very basic example)
def parse_log(file_path):
    with open(file_path, 'r') as file:
        for line in file:
            if "http://" in line or "https://" in line:
                # Extract the URL from the log line
                url = extract_url_from_log_line(line)
                print(f"URL found in log: {url}")
                # Analyze the URL (send it to an API like VirusTotal)
                analyze_url(url_analysis)

def extract_url_from_log_line(line):
   # A very basic URL extraction, depending on the log format
    url_start = line.find("http://") if "http://" in line else line.find("https://")
    url_end = line.find(" ", url_start)
    return line[url_start:url_end]

# Example usage
log_file_path = "/path/to/your/logfile.log"
parse_log(log_file_path)

load_dotenv()  # Load environment variables from .env file

API_KEY = os.getenv('ALIENVAULT_OTX_API_KEY')

if API_KEY is None:
    raise ValueError("API Key not found in .env file.")

echo ".env" >> .gitignore

# Use API_KEY in your OTXv2 initialization
otx = OTXv2.OTXv2(API_KEY)

def fetch_threat_data_from_otx(url_to_analyze):
    try:
        # Use the OTX API to get information about the URL
        pulse_info = otx.get_indicator_details_full(otx.IndicatorTypes.URL, url_to_analyze)
        
        # Extract relevant data from the response
        threat_info = {
            "url": url_to_analyze,
            "pulse_info": pulse_info
        }
        
        return threat_info
    
    except Exception as e:
        print(f"An error occurred while fetching threat data: {e}")
        return None

# Example usage
url_to_analyze = [url_analysis]  # Replace with the actual URL
threat_data = fetch_threat_data_from_otx(url_to_analyze)
if threat_data:
    print(f"Fetched threat data: {json.dumps(threat_data, indent=4)}

# Block the given IP address using iptables.
# Define block ip function
def block_ip(ip_address):
    
# Args:
        ip_address (str): [ip_src]  #  The IP address to block.
    
try:
        # Use subprocess to run the iptables command
    subprocess.run(["sudo", "iptables", "-A", "INPUT", "-s", ip_address, "-j", "DROP"], check=True)
    print(f"Successfully blocked IP address: {ip_address}")
except subprocess.CalledProcessError as e:
    print(f"Failed to block IP address {ip_address}: {e}")
except Exception as e:
    print(f"An unexpected error occurred while blocking IP address: {e}")


    # Log the firewall update for the blocked IP address.
# define log firewall update funtion
def log_firewall_update(ip_address):
    
# Args:
        ip_address (str): [ip_src]  # The IP address that was blocked.
    
try:
    with open("firewall_updates.log", "a") as log_file:
        log_file.write(f"{time.ctime()}: Blocked IP {ip_address}\n")
    print(f"Logged firewall update for IP: {ip_address}")
except IOError as e:
    print(f"Failed to write to log file: {e}")
except Exception as e:
    print(f"An unexpected error occurred while logging firewall update: {e}")

# Process the threat data
api_url = 'https://api.threatintelligenceplatform.com/v1/threat-data'  # Replace with an actual API
threat_data = fetch_threat_feed(api_url)
if threat_data:
    print(f"Fetched threat data: {threat_data}")
    
    # Process the threat data
    for threat in threat_data:
        # Assume the threat data contains IP addresses to block
        if 'ip' in threat:
            ip_address = threat['ip']
            print(f"Processing threat with IP: {ip_address}")

            # Block the IP using the firewall
            block_ip(ip_address)

            # Log the action
            log_firewall_update(ip_address)
            
        # If the threat data includes domain names or other indicators
        if 'domain' in threat:
            domain = threat['domain']
            print(f"Processing threat with domain: {domain}")
            # You could implement domain-specific blocking or monitoring here
            # Example: You could add the domain to a blacklist, or log the domain for further analysis

    # Implement logic to block the URL
    # This might involve adding it to a web proxy or firewall rule
    # Example: You might add this URL to a firewall rule or web proxy blacklist
    # This is a placeholder for the actual implementation

def block_url(url):
    """
    Block the given URL using iptables by resolving it to its IP address and blocking that IP.
    
    Args:
        url (str): The URL to block.
    """
    try:
        # Resolve the URL to its IP address
        ip_address = socket.gethostbyname(url)
        
        # Block the IP address using iptables
        subprocess.run(["sudo", "iptables", "-A", "INPUT", "-s", ip_address, "-j", "DROP"], check=True)
        
        print(f"Successfully blocked URL: {url} (resolved to IP: {ip_address})")
        
        # Log the firewall update
        log_firewall_update(ip_address)
        
    except socket.gaierror:
        print(f"Failed to resolve URL: {url}")
    except subprocess.CalledProcessError as e:
        print(f"Failed to block IP address for URL {url}: {e}")
    except Exception as e:
        print(f"An unexpected error occurred while blocking URL: {e}")

# Block access to the URL by adding it to the DNS blocklist.
def block_url(url):
   
# Args:
        url (str): [str_src]  # The URL to block.
   
domain = extract_domain_from_url(url_analysis)
    
try:
    # Example of adding the domain to a DNS blackhole list
    with open("/etc/hosts", "a") as hosts_file:
        hosts_file.write(f"127.0.0.1 {domain}\n")
    print(f"Successfully blocked URL: {url} by adding domain {domain} to DNS blocklist.")
except IOError as e:
    print(f"Failed to block URL {url}: {e}")
except Exception as e:
    print(f"An unexpected error occurred while blocking URL: {e}")

# Extracts the domain from a given URL.
def extract_domain_from_url(url):

# Args:
        url (str): [url_analysis]  # The URL from which to extract the domain.

# Returns:
        str: [domain]  # The extracted domain.

        from urllib.parse import urlparse
        parsed_url = urlparse(url)
        return parsed_url.netloc

def log_url_block(url):

# Log the blocked URL for auditing purposes.
#Args:
    url (str): [url_analysis]  # The URL that was blocked.
    
    try:
        with open("url_blocks.log", "a") as log_file:
            log_file.write(f"{time.ctime()}: Blocked URL {url}\n")
    except IOError as e:
        print(f"Failed to write to log file: {e}")
    except Exception as e:
        print(f"An unexpected error occurred while logging URL block: {e}")

# Update the firewall to block traffic based on the detected threat.
def update_firewall(packet=None, url=None):
    
# Args:
        packet (object): [ip_src]  #  The packet data related to the threat (optional).
        url (str): [url_analysis]  # The URL to block (optional).
    
if packet:
    # Placeholder logic for firewall update based on packet
    src_ip = packet[0][1].src
    block_ip(ip_src)
    print(f"Updating firewall to block packet from IP: {src_ip}")
    
    if url:
        # Use the block_url function to block the URL
        block_url(url_analysis)

# Example usage:
if __name__ == "__main__":
    # Example threat data
    threat_data = {
        'packet': [ip_src],
            url: [url_analysis]  # Replace with an actual URL you want to block
            threat_level: 'high'
            }
    
    # Run the firewall update process based on the threat data
    if 'url' in threat_data:
        update_firewall(url=threat_data[url_analysis])
    if 'packet' in threat_data:
        update_firewall(packet=threat_data['packet'])

def log_url_block(url):
    # Log the blocked URL for auditing purposes
    try:
        with open("url_blocks.log", "a") as log_file:
            log_file.write(f"{time.ctime()}: Blocked URL {url}\n")
    except IOError as e:
        print(f"Failed to write to log file: {e}")
    except Exception as e:
        print(f"An unexpected error occurred while logging URL block: {e}")

def blacklist_domain(domain):
    # Implement logic to add the domain to a blacklist
    # Example: You might add this domain to a DNS filter or firewall rule
    print(f"Blacklisting domain: {domain}")
    # This is a placeholder for the actual implementation

def log_domain_blacklist(domain):
    # Log the blacklisted domain for auditing purposes
    with open("domain_blacklists.log", "a") as log_file:
        log_file.write(f"{time.ctime()}: Blacklisted domain {domain}\n")

def insert_file_hash(file_hash, file_path, db_name='lobsterpot.db'):
    conn = sqlite3.connect(db_name)
    cursor = conn.cursor()
    timestamp = datetime.now().isoformat()
    
    try:
        cursor.execute('''INSERT OR IGNORE INTO file_hashes (file_hash, file_path, timestamp)
                          VALUES (?, ?, ?)''', (file_hash, file_path, timestamp))
        conn.commit()
        print(f"Inserted file hash: {file_hash} for file: {file_path}")
    except sqlite3.DatabaseError as db_err:
        print(f"Database error occurred: {db_err}")
    except Exception as e:
        print(f"An unexpected error occurred while inserting file hash: {e}")
    finally:
        if conn:
            conn.close()

# Example threat data containing a domain
threat_data = {
    'domain': 'malicious-example.com'
}

# Process the domain threat
if 'domain' in threat_data:
    domain = threat_data['domain']
    print(f"Processing threat with domain: {domain}")
    blacklist_domain(domain)  # Add the domain to the blacklist
    log_domain_blacklist(domain)  # Log the action

# Implement logic to check if the file hash is present on the system
# Example: You might search the filesystem or a database of known hashes
# This is a placeholder for the actual implementation
def is_file_hash_present(file_hash, db_name='lobsterpot.db'):
    conn = sqlite3.connect(db_name)
    cursor = conn.cursor()
    
    try:
        cursor.execute('''SELECT COUNT(*) FROM file_hashes WHERE file_hash = ?''', (file_hash,))
        result = cursor.fetchone()[0]
        return result > 0  # Return True if the file hash is found, otherwise False
    except sqlite3.DatabaseError as db_err:
        print(f"Database error occurred: {db_err}")
        return False  # Example: Return True if the file is found, otherwise False
    except Exception as e:
        print(f"An unexpected error occurred while checking file hash: {e}")
        return False
    finally:
        if conn:
            conn.close()

    # Check if the file hash is present
    if is_file_hash_present(example_file_hash):
        print(f"File hash {example_file_hash} is present in the database.")
    else:
        print(f"File hash {example_file_hash} is NOT present in the database.")

    # Implement logic to quarantine the file with the given hash
    # Example: You might move the file to a secure location or delete it
    with open("file_quarantines.log", "a") as log_file:
        log_file.write(f"{time.ctime()}: Quarantined file with hash {file_hash}\n")

# extending it to handle other types of IoCs such as URLs and file hashes
api_url = 'https://api.threatintelligenceplatform.com/v1/threat-data'  # Replace with an actual API
threat_data = fetch_threat_feed(api_url)
if threat_data:
    print(f"Fetched threat data: {threat_data}")
    
    # Process the threat data
    for threat in threat_data:
        # Handle IP addresses
        if 'ip' in threat:
            ip_address = threat['ip']
            print(f"Processing threat with IP: {ip_address}")

            # Block the IP using the firewall
            block_ip(ip_address)

            # Log the action
            log_firewall_update(ip_address)

        # Handle domain names
        if 'domain' in threat:
            domain = threat['domain']
            print(f"Processing threat with domain: {domain}")
            # Example: Add the domain to a blacklist (implementation needed)

def quarantine_file(file_hash, quarantine_dir="/path/to/quarantine", db_name='lobsterpot.db'):

    conn = sqlite3.connect(db_name)
    cursor = conn.cursor()

    print(f"Quarantining file with hash: {file_hash}")
    This is a placeholder for the actual implementation
    Locate the file by its hash
    try:
    cursor.execute('''SELECT file_path FROM file_hashes WHERE file_hash = ?''', (file_hash,))
    file_path = cursor.fetchone()

    if file_path is None:
         print(f"No file found with hash {file_hash} in the database.")
        return False
        
file_path = file_path[0]  # Extract the file path from the tuple
        
if not os.path.exists(file_path):
        print(f"File {file_path} does not exist.")
        return False
        
    # Move the file to the quarantine directory
        quarantine_path = os.path.join(quarantine_dir, os.path.basename(file_path))
        shutil.move(file_path, quarantine_path)
        print(f"File {file_path} quarantined to {quarantine_path}.")