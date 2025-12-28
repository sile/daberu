# Sora Python SDK Examples

## Complete Sendonly Example

```python
import json
import time
import cv2
import numpy as np
import sounddevice as sd
from threading import Event
from sora_sdk import Sora, SoraSignalingErrorCode

class SendonlyExample:
    def __init__(self, signaling_urls, channel_id):
        self.signaling_urls = signaling_urls
        self.channel_id = channel_id
        self.connected = Event()
        
        # Create Sora instance
        self.sora = Sora()
        
        # Create audio/video sources
        self.audio_source = self.sora.create_audio_source(
            channels=1, 
            sample_rate=16000
        )
        self.video_source = self.sora.create_video_source()
        
        # Create connection
        self.connection = self.sora.create_connection(
            signaling_urls=signaling_urls,
            role="sendonly",
            channel_id=channel_id,
            audio_source=self.audio_source,
            video_source=self.video_source
        )
        
        # Set up callbacks
        self.connection.on_set_offer = self.on_set_offer
        self.connection.on_notify = self.on_notify
        self.connection.on_disconnect = self.on_disconnect
        
    def on_set_offer(self, raw_message):
        message = json.loads(raw_message)
        if message["type"] == "offer":
            self.connection_id = message["connection_id"]
            print(f"Connection ID: {self.connection_id}")
    
    def on_notify(self, raw_message):
        message = json.loads(raw_message)
        if (message["type"] == "notify" and 
            message["event_type"] == "connection.created" and
            message["connection_id"] == self.connection_id):
            print("Connected!")
            self.connected.set()
    
    def on_disconnect(self, error_code, message):
        print(f"Disconnected: {error_code} - {message}")
        self.connected.clear()
    
    def audio_callback(self, indata, frames, time_info, status):
        # Send audio from microphone
        self.audio_source.on_data(indata)
    
    def run(self):
        # Start audio input
        audio_stream = sd.InputStream(
            samplerate=16000,
            channels=1,
            dtype='int16',
            callback=self.audio_callback
        )
        
        # Start video capture
        cap = cv2.VideoCapture(0)
        
        # Connect
        self.connection.connect()
        assert self.connected.wait(10), "Failed to connect"
        
        with audio_stream:
            try:
                while self.connected.is_set():
                    ret, frame = cap.read()
                    if ret:
                        self.video_source.on_captured(frame)
                    time.sleep(1/30)  # 30 FPS
            except KeyboardInterrupt:
                pass
            finally:
                self.connection.disconnect()
                cap.release()

# Usage
example = SendonlyExample(
    signaling_urls=["wss://sora.example.com/signaling"],
    channel_id="my-channel"
)
example.run()
```

## Complete Recvonly Example

```python
import json
import time
import cv2
import numpy as np
import sounddevice as sd
from threading import Event, Lock
from sora_sdk import Sora, SoraAudioSink, SoraVideoSink

class RecvonlyExample:
    def __init__(self, signaling_urls, channel_id):
        self.signaling_urls = signaling_urls
        self.channel_id = channel_id
        self.connected = Event()
        
        self.audio_sink = None
        self.video_sinks = {}
        self.video_frames = {}
        self.frames_lock = Lock()
        
        # Create Sora instance
        self.sora = Sora()
        
        # Create connection
        self.connection = self.sora.create_connection(
            signaling_urls=signaling_urls,
            role="recvonly",
            channel_id=channel_id
        )
        
        # Set up callbacks
        self.connection.on_set_offer = self.on_set_offer
        self.connection.on_notify = self.on_notify
        self.connection.on_disconnect = self.on_disconnect
        self.connection.on_track = self.on_track
        
    def on_set_offer(self, raw_message):
        message = json.loads(raw_message)
        if message["type"] == "offer":
            self.connection_id = message["connection_id"]
    
    def on_notify(self, raw_message):
        message = json.loads(raw_message)
        if (message["type"] == "notify" and 
            message["event_type"] == "connection.created" and
            message["connection_id"] == self.connection_id):
            print("Connected!")
            self.connected.set()
    
    def on_disconnect(self, error_code, message):
        print(f"Disconnected: {error_code} - {message}")
        self.connected.clear()
    
    def on_track(self, track):
        if track.kind == "audio":
            self.audio_sink = SoraAudioSink(
                track, 
                output_frequency=16000,
                output_channels=1
            )
            print("Audio track received")
            
        elif track.kind == "video":
            video_sink = SoraVideoSink(track)
            
            def on_frame(frame):
                with self.frames_lock:
                    self.video_frames[track.id] = frame.data().copy()
            
            video_sink.on_frame = on_frame
            self.video_sinks[track.id] = video_sink
            print(f"Video track received: {track.id}")
    
    def audio_callback(self, outdata, frames, time_info, status):
        if self.audio_sink:
            success, data = self.audio_sink.read(frames)
            if success:
                outdata[:] = data
    
    def run(self):
        # Start audio output
        audio_stream = sd.OutputStream(
            samplerate=16000,
            channels=1,
            dtype='int16',
            callback=self.audio_callback
        )
        
        # Connect
        self.connection.connect()
        assert self.connected.wait(10), "Failed to connect"
        
        with audio_stream:
            try:
                while self.connected.is_set():
                    # Display received video
                    with self.frames_lock:
                        for track_id, frame in self.video_frames.items():
                            cv2.imshow(f'Received: {track_id}', frame)
                    
                    if cv2.waitKey(1) & 0xFF == ord('q'):
                        break
                        
                    time.sleep(0.01)
            except KeyboardInterrupt:
                pass
            finally:
                self.connection.disconnect()
                cv2.destroyAllWindows()

# Usage
example = RecvonlyExample(
    signaling_urls=["wss://sora.example.com/signaling"],
    channel_id="my-channel"
)
example.run()
```

## Messaging Example

```python
import json
from threading import Event
from sora_sdk import Sora

class MessagingExample:
    def __init__(self, signaling_urls, channel_id):
        self.connected = Event()
        self.data_channel_ready = Event()
        
        self.sora = Sora()
        self.connection = self.sora.create_connection(
            signaling_urls=signaling_urls,
            role="sendrecv",
            channel_id=channel_id,
            audio=False,
            video=False,
            data_channel_signaling=True,
            data_channels=[
                {"label": "#messaging", "direction": "sendrecv"}
            ]
        )
        
        self.connection.on_notify = self.on_notify
        self.connection.on_data_channel = self.on_data_channel
        self.connection.on_message = self.on_message
        
    def on_notify(self, raw_message):
        message = json.loads(raw_message)
        if message.get("event_type") == "connection.created":
            self.connected.set()
    
    def on_data_channel(self, label):
        print(f"Data channel ready: {label}")
        self.data_channel_ready.set()
    
    def on_message(self, label, data):
        print(f"Received on {label}: {data.decode('utf-8')}")
    
    def send(self, message):
        self.data_channel_ready.wait()
        self.connection.send_data_channel("#messaging", message.encode('utf-8'))
    
    def run(self):
        self.connection.connect()
        self.connected.wait(10)
        
        try:
            while True:
                msg = input("Enter message: ")
                self.send(msg)
        except KeyboardInterrupt:
            self.connection.disconnect()

# Usage
example = MessagingExample(
    signaling_urls=["wss://sora.example.com/signaling"],
    channel_id="my-channel"
)
example.run()
```

## VAD Example

```python
from sora_sdk import Sora, SoraVAD, SoraAudioStreamSink

class VADExample:
    def __init__(self, signaling_urls, channel_id):
        self.vad = SoraVAD()
        
        self.sora = Sora()
        self.connection = self.sora.create_connection(
            signaling_urls=signaling_urls,
            role="recvonly",
            channel_id=channel_id,
            audio=True,
            video=False
        )
        
        self.connection.on_track = self.on_track
    
    def on_track(self, track):
        if track.kind == "audio":
            audio_sink = SoraAudioStreamSink(track, 16000, 1)
            audio_sink.on_frame = self.on_audio_frame
    
    def on_audio_frame(self, frame):
        probability = self.vad.analyze(frame)
        if probability > 0.95:
            print(f"Voice detected! (probability: {probability:.2f})")
    
    def run(self):
        self.connection.connect()
        try:
            while True:
                pass  # Audio processing happens in callback
        except KeyboardInterrupt:
            self.connection.disconnect()

# Usage
example = VADExample(
    signaling_urls=["wss://sora.example.com/signaling"],
    channel_id="my-channel"
)
example.run()
```

## Hardware Acceleration Example

```python
from sora_sdk import (
    Sora,
    SoraVideoCodecPreference,
    SoraVideoCodecType,
    SoraVideoCodecImplementation,
    get_video_codec_capability
)

# Get available hardware acceleration
capability = get_video_codec_capability()

for engine in capability.engines:
    print(f"Engine: {engine.name}")
    for codec in engine.codecs:
        print(f"  {codec.type}: encoder={codec.encoder}, decoder={codec.decoder}")

# Use Intel VPL for H264
preference = SoraVideoCodecPreference(
    codecs=[
        SoraVideoCodecPreference.Codec(
            type=SoraVideoCodecType.H264,
            encoder=SoraVideoCodecImplementation.INTEL_VPL,
            decoder=SoraVideoCodecImplementation.INTEL_VPL
        )
    ]
)

sora = Sora(video_codec_preference=preference)

# Or use helper to get all codecs from specific implementation
from sora_sdk import create_video_codec_preference_from_implementation

preference = create_video_codec_preference_from_implementation(
    capability,
    SoraVideoCodecImplementation.INTEL_VPL
)

sora = Sora(video_codec_preference=preference)
```

## Simulcast Example

```python
from sora_sdk import Sora

sora = Sora()
video_source = sora.create_video_source()

connection = sora.create_connection(
    signaling_urls=["wss://sora.example.com/signaling"],
    role="sendonly",
    channel_id="my-channel",
    simulcast=True,
    video_codec_type="VP9",
    video_bit_rate=5550,  # Total for all layers
    video_source=video_source
)

connection.connect()

# Check simulcast in stats
import json
stats = json.loads(connection.get_stats())
for stat in stats:
    if stat.get("type") == "outbound-rtp" and stat.get("kind") == "video":
        print(f"RID: {stat.get('rid')}, Bitrate: {stat.get('targetBitrate')}")
```

## Encoded Transform Example

```python
import numpy as np
from sora_sdk import Sora, SoraVideoFrameTransformer

class EncryptedSender:
    def __init__(self, signaling_urls, channel_id):
        self.transformer = SoraVideoFrameTransformer()
        self.transformer.on_transform = self.encrypt_frame
        
        sora = Sora()
        video_source = sora.create_video_source()
        
        self.connection = sora.create_connection(
            signaling_urls=signaling_urls,
            role="sendonly",
            channel_id=channel_id,
            video_source=video_source,
            video_frame_transformer=self.transformer
        )
    
    def encrypt_frame(self, frame):
        # Get encoded data
        data = frame.get_data()
        
        # Simple XOR encryption (use real crypto in production!)
        data_array = np.asarray(data, dtype=np.uint8)
        encrypted = data_array ^ 0xAA  # XOR with key
        
        # Set modified data
        frame.set_data(encrypted)
        
        # Return to stream
        self.transformer.enqueue(frame)

class EncryptedReceiver:
    def __init__(self, signaling_urls, channel_id):
        sora = Sora()
        
        self.connection = sora.create_connection(
            signaling_urls=signaling_urls,
            role="recvonly",
            channel_id=channel_id
        )
        
        self.connection.on_track = self.on_track
    
    def on_track(self, track):
        if track.kind == "video":
            transformer = SoraVideoFrameTransformer()
            transformer.on_transform = self.decrypt_frame
            track.set_frame_transformer(transformer)
            self.transformer = transformer
    
    def decrypt_frame(self, frame):
        # Get encoded data
        data = frame.get_data()
        
        # Decrypt (XOR again to reverse)
        data_array = np.asarray(data, dtype=np.uint8)
        decrypted = data_array ^ 0xAA
        
        # Set modified data
        frame.set_data(decrypted)
        
        # Return to stream
        self.transformer.enqueue(frame)
```
