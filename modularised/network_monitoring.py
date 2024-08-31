from scapy.all import sniff

def monitor_traffic(interface="eth0"):
    """Monitor network traffic and detect threats."""
    print(f"Monitoring traffic on {interface}...")
    sniff(iface=interface, prn=process_packet, store=False)


# Function to process captured packets
def process_packet(packet):
    # Extract features from the packet for analysis
    try:
        features = extract_features(packet)
        is_threat = detect_threat(features)
        if is_threat:
            print(f"Threat detected: {packet.summary()}")
            update_firewall(packet)
    except KeyError as e:
        print(f"Key error in packet processing: {e}")
    except Exception as e:
        print(f"An unexpected error occurred while processing packet: {e}")
        # Trigger adaptive firewall update
        update_firewall(packet)

    logging.info(f"Processing packet: {packet.summary()}")
# More network monitoring functions...



# Function to fetch threat intelligence data from an external threat feed
def fetch_threat_feed(api_url, retries=3, delay=5):
    attempt = 0
    while attempt < retries:
        try:
            response = requests.get(api_url, timeout=10)  # Adding a timeout for better control
            response.raise_for_status()  # Raises an HTTPError if the status is not 200
            threat_data = response.json()
            for threat in threat_data:
                block_ip(threat["ip"])
            return threat_data
        except (Timeout, HTTPError, RequestException) as e:
            print(f"Attempt {attempt + 1} failed: {e}")
            attempt += 1
            if attempt < retries:
                print(f"Retrying in {delay} seconds...")
                time.sleep(delay)
        except Exception as e:
            print(f"An unexpected error occurred: {e}")
            break
    print("Failed to fetch threat feed after several attempts.")
    return None

# Extract features from the packet for analysis
def extract_features(packet):

    # Initialize a dictionary to hold the features
    features = {}

    # Basic IP layer features
    if packet.haslayer(IP):
        features['src_ip'] = packet[IP].src
        features['dst_ip'] = packet[IP].dst
        features['ttl'] = packet[IP].ttl
        features['ip_len'] = packet[IP].len
        features['proto'] = packet[IP].proto

    # TCP layer features
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
