# network_monitoring.py

import time
import logging
import requests
from scapy.all import sniff
from scapy.layers.inet import IP, TCP, UDP, ICMP
from scapy.layers.http import HTTPRequest
from requests.exceptions import RequestException, Timeout, HTTPError


# Initialize logging
logging.basicConfig(level=logging.INFO)

def monitor_traffic(interface="eth0"):
    """Monitor network traffic and detect threats."""
    logging.info(f"Monitoring traffic on {interface}...")
    try:
        sniff(iface=interface, prn=process_packet, store=False)
    except Exception as e:
        logging.error(f"Error during network monitoring: {e}")

def process_packet(packet):
    """Process captured packets for analysis."""
    try:
        features = extract_features(packet)
        is_threat = detect_threat(features)
        if is_threat:
            logging.warning(f"Threat detected: {packet.summary()}")
            update_firewall(packet)
    except KeyError as e:
        logging.error(f"Key error in packet processing: {e}")
    except Exception as e:
        logging.error(f"Unexpected error while processing packet: {e}")
        update_firewall(packet)

def fetch_threat_feed(api_url, retries=3, delay=5):
    """Fetch threat intelligence data from an external threat feed."""
    attempt = 0
    while attempt < retries:
        try:
            response = requests.get(api_url, timeout=10)
            response.raise_for_status()  # Raises an HTTPError if the status is not 200
            threat_data = response.json()
            for threat in threat_data:
                block_ip(threat["ip"])
            return threat_data
        except (Timeout, HTTPError, RequestException) as e:
            logging.warning(f"Attempt {attempt + 1} failed: {e}")
            attempt += 1
            if attempt < retries:
                logging.info(f"Retrying in {delay} seconds...")
                time.sleep(delay)
        except Exception as e:
            logging.error(f"Unexpected error while fetching threat feed: {e}")
            break
    logging.error("Failed to fetch threat feed after several attempts.")
    return None

def extract_features(packet):
    """Extract features from the packet for analysis."""
    features = {}
    # Add checks to ensure layers are present in packet
    if packet.haslayer(IP):
        features['src_ip'] = packet[IP].src
        features['dst_ip'] = packet[IP].dst
        features['ttl'] = packet[IP].ttl
        features['ip_len'] = packet[IP].len
        features['proto'] = packet[IP].proto

    # Continue adding features based on presence
    if packet.haslayer(TCP):
        features['src_port'] = packet[TCP].sport
        features['dst_port'] = packet[TCP].dport
        features['tcp_flags'] = packet[TCP].flags
        features['tcp_seq'] = packet[TCP].seq
        features['tcp_ack'] = packet[TCP].ack

    # UDP layer features
    if packet.haslayer(UDP):
        features['src_port'] = packet[UDP].sport
        features['dst_port'] = packet[UDP].dport
        features['udp_len'] = packet[UDP].len

    # ICMP layer features
    if packet.haslayer(ICMP):
        features['icmp_type'] = packet[ICMP].type
        features['icmp_code'] = packet[ICMP].code

    # HTTP layer features (if applicable)
    if packet.haslayer(HTTPRequest):
        features['http_method'] = packet[HTTPRequest].Method.decode()
        features['http_host'] = packet[HTTPRequest].Host.decode()
        features['http_path'] = packet[HTTPRequest].Path.decode()

    # General features
    features['packet_length'] = len(packet)  # Total packet length
    features['timestamp'] = packet.time  # Timestamp of the packet capture

    return features

from sklearn.ensemble import IsolationForest
import numpy as np

def detect_threat(features, model):
    """Detect if the features indicate a threat using a trained model.

    Args:
        features (dict): Features extracted from the packet.
        model (IsolationForest): Trained anomaly detection model.

    Returns:
        bool: True if threat is detected, False otherwise.
    """
    try:
        input_data = np.array([[features['length'], features['protocol']]])
        prediction = model.predict(input_data)
        return prediction[0] == -1  # IsolationForest returns -1 for anomalies
    except KeyError as e:
        log_message(f"Key error in feature extraction: {e}")
        return False
    except Exception as e:
        log_message(f"Unexpected error during threat detection: {e}")
        return False

def update_firewall(packet):
    """Placeholder for firewall update logic."""
    # Implement firewall update logic
    pass

def block_ip(ip_address):
    """Placeholder for IP blocking logic."""
    # Implement IP blocking logic
    pass
