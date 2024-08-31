def train_isolation_forest(X_train_scaled):
    """Train Isolation Forest for anomaly detection."""
    isolation_forest = IsolationForest(n_estimators=100, contamination=0.05, random_state=42)
    isolation_forest.fit(X_train_scaled)
    return isolation_forest

def evaluate_model(isolation_forest, X_test_scaled, y_test):
    """Evaluate the Isolation Forest model."""
    y_pred = isolation_forest.predict(X_test_scaled)
    y_pred = np.where(y_pred == -1, 1, 0)
    print("Confusion Matrix:")
    print(confusion_matrix(y_test, y_pred))
    print("\nClassification Report:")
    print(classification_report(y_test, y_pred))


# Simulate normal behavior data (e.g., CPU usage, memory usage, network traffic)
normal_data = np.random.normal(loc=50, scale=5, size=(1000, 3))  # Normal data
malicious_data = np.random.normal(loc=80, scale=10, size=(50, 3))  # Simulated malicious data

# Combine into one dataset
data = np.vstack((normal_data, malicious_data))
labels = np.array([0] * 1000 + [1] * 50)  # 0 = normal, 1 = malicious

# Convert to a DataFrame for easier manipulation
df = pd.DataFrame(data, columns=['cpu_usage', 'memory_usage', 'network_traffic'])
df['label'] = labels

# Split the data into training and test sets
X_train, X_test, y_train, y_test = train_test_split(df[['cpu_usage', 'memory_usage', 'network_traffic']], df['label'], test_size=0.2, random_state=42)


def build_rnn(input_shape):
    rnn_model = Sequential([
        SimpleRNN(64, input_shape=input_shape, return_sequences=True),
        Dense(32, activation='relu'),
        Dense(1, activation='sigmoid')
    ])
    return rnn_model

def build_cnn(input_shape):
    cnn_model = Sequential([
        Conv1D(32, kernel_size=3, activation='relu', input_shape=input_shape),
        MaxPooling1D(pool_size=2),
        Flatten(),
        Dense(64, activation='relu'),
        Dense(1, activation='sigmoid')
    ])
    return cnn_model

def build_lstm(input_shape):
    lstm_model = Sequential([
        LSTM(64, input_shape=input_shape, return_sequences=True),
        Dense(32, activation='relu'),
        Dense(1, activation='sigmoid')
    ])
    return lstm_model

class CentralGanglion(tf.keras.Model):
    def __init__(self, rnn, cnn, lstm):
        super(CentralGanglion, self).__init__()
        self.rnn = rnn
        self.cnn = cnn
        self.lstm = lstm
        self.dense = Dense(3, activation='relu')
        self.output_layer = Dense(1, activation='sigmoid')

    def call(self, rnn_input, cnn_input, lstm_input):
        rnn_output = self.rnn(rnn_input)
        cnn_output = self.cnn(cnn_input)
        lstm_output = self.lstm(lstm_input)

        combined_output = tf.concat([rnn_output[:, -1], cnn_output, lstm_output[:, -1]], axis=-1)
        ganglion_output = self.dense(combined_output)
        return self.output_layer(ganglion_output)


def incremental_training(isolation_forest, ganglion, X_train, y_train, interval=300):
    while True:
        isolation_forest.fit(X_train)

        y_pred_updated = isolation_forest.predict(X_test_scaled)
        y_pred_updated = np.where(y_pred_updated == -1, 1, 0)

        print("\nUpdated Confusion Matrix:")
        print(confusion_matrix(y_test, y_pred_updated))
        print("\nUpdated Classification Report:")
        print(classification_report(y_test, y_pred_updated))

        history = ganglion.fit([X_train] * 3, y_train, epochs=10, batch_size=8, validation_split=0.2)

        time.sleep(interval)
