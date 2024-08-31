import requests
from requests.exceptions import RequestException, Timeout, HTTPError
import time
import subprocess
import logging
import socket

# Initialize logging
logging.basicConfig(level=logging.INFO)

def fetch_threat_feed(api_url, retries=3, delay=5):
    """Fetch threat intelligence data from an external threat feed."""
    attempt = 0
    while attempt < retries:
        try:
            response = requests.get(api_url, timeout=10)
            response.raise_for_status()
            threat_data = response.json()
            for threat in threat_data:
                if 'ip' in threat:
                    block_ip(threat["ip"])
            return threat_data
        except (Timeout, HTTPError, RequestException) as e:
            logging.error(f"Attempt {attempt + 1} failed: {e}")
            attempt += 1
            if attempt < retries:
                logging.info(f"Retrying in {delay} seconds...")
                time.sleep(delay)
        except Exception as e:
            logging.error(f"An unexpected error occurred: {e}")
            break
    logging.error("Failed to fetch threat feed after several attempts.")
    return None

def block_ip(ip_address):
    """Block the given IP address using iptables."""
    try:
        subprocess.run(["sudo", "iptables", "-A", "INPUT", "-s", ip_address, "-j", "DROP"], check=True)
        logging.info(f"Successfully blocked IP address: {ip_address}")
    except subprocess.CalledProcessError as e:
        logging.error(f"Failed to block IP address {ip_address}: {e}")
    except Exception as e:
        logging.error(f"An unexpected error occurred while blocking IP address: {e}")

def block_url(url):
    """Block the given URL by resolving its IP address and using iptables."""
    try:
        ip_address = socket.gethostbyname(url)
        block_ip(ip_address)
        logging.info(f"Successfully blocked URL: {url} (resolved to IP: {ip_address})")
    except socket.gaierror:
        logging.error(f"Failed to resolve URL: {url}")
    except Exception as e:
        logging.error(f"An unexpected error occurred while blocking URL: {e}")

def update_firewall(packet=None, url=None):
    """Update the firewall to block traffic based on the detected threat."""
    if packet:
        src_ip = packet.get('src_ip')
        if src_ip:
            block_ip(src_ip)
            logging.info(f"Firewall updated to block IP from packet: {src_ip}")
        else:
            logging.warning("No source IP found in the packet.")
    
    if url:
        block_url(url)
        logging.info(f"Firewall updated to block traffic from URL: {url}")

# Example usage
if __name__ == "__main__":
    api_url = 'https://api.threatintelligenceplatform.com/v1/threat-data'  # Replace with an actual API URL
    threat_data = fetch_threat_feed(api_url)
    if threat_data:
        logging.info(f"Fetched threat data: {threat_data}")
